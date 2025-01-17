use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::token;

use super::RegInfo;
use super::{CodegenContext, OpCategory, ToTokenStream};
use crate::ast::{Expression, TypedExpression};
use crate::SpannedError;

use rvv_assembler::{Imm, Ivi, Ivv, VConfig, VInst, VReg, Vlmul, Vtypei, XReg};

impl CodegenContext {
    // Generate raw asm statements for top level expression
    pub(crate) fn gen_tokens(
        &mut self,
        expr: &TypedExpression,
        top_level: bool,
        extra_bind_id: Option<usize>,
        exists_vd: Option<u8>,
        mut bit_length: u16,
    ) -> Result<TokenStream, SpannedError> {
        let (left, op, right, is_assign) = match &expr.expr.0 {
            Expression::Assign { left, right, .. } => {
                if let Some(var_ident) = left.expr.0.var_ident() {
                    if let Some(vd) = self.var_regs.get(var_ident).cloned() {
                        return self.gen_tokens(right, true, None, Some(vd), bit_length);
                    }
                }
                let mut tokens = TokenStream::new();
                left.to_tokens(&mut tokens, self)?;
                token::Eq::default().to_tokens(&mut tokens);
                right.to_tokens(&mut tokens, self)?;
                return Ok(tokens)
            }
            Expression::AssignOp { left, op, right } => (left, op, right, true),
            Expression::Binary { left, op, right } => (left, op, right, false),
            Expression::Path(path) => {
                let mut tokens = TokenStream::new();
                if top_level {
                    // Here assume this expression is the return value of current block
                    if let Some(var_ident) = path.get_ident() {
                        if let Some(vreg) = self.var_regs.get(var_ident).cloned() {
                            for RegInfo{number, bit_length, ..} in self.expr_regs.values() {
                                if *number == vreg {
                                    vstore_codegen(&mut tokens, vreg, *bit_length, self.show_asm);
                                    let mut rv = TokenStream::new();
                                    token::Brace::default().surround(&mut rv, |inner| {
                                        inner.extend(Some(tokens));
                                    });
                                    return Ok(rv);
                                }
                            }
                        }
                    }
                }
                path.to_tokens(&mut tokens);
                return Ok(tokens);
            }
            Expression::MethodCall { receiver, method, args, .. } => {
                return self.gen_method_call_tokens(expr, receiver, method, args, top_level, extra_bind_id, bit_length);
            }
            Expression::Paren { expr: sub_expr, .. } => {
                return self.gen_tokens(&*sub_expr, top_level, Some(expr.id), exists_vd, bit_length);
            }
            _  => return Err((expr.expr.1, anyhow!("invalid expression, inner expression must be simple variable name or binary op"))),
        };
        if !top_level && is_assign {
            return Err((
                expr.expr.1,
                anyhow!("assign op in sub-expression is forbidden"),
            ));
        }

        let mut tokens = TokenStream::new();

        if top_level {
            let left_type_name = left.type_name();
            let right_type_name = right.type_name();
            match (left_type_name.as_deref(), right_type_name.as_deref()) {
                (Some("U256"), Some("U256")) => {
                    bit_length = 256;
                }
                (Some("U512"), Some("U512")) => {
                    bit_length = 512;
                }
                (Some("U1024"), Some("U1024")) => {
                    bit_length = 1024;
                }
                _ => {
                    left.to_tokens(&mut tokens, self)?;
                    op.to_tokens(&mut tokens);
                    right.to_tokens(&mut tokens, self)?;
                    return Ok(tokens);
                }
            };
        }

        self.update_vconfig(&mut tokens, bit_length);
        self.gen_sub_exprs(&mut tokens, left, right, bit_length)?;

        let op_category = OpCategory::from(op);
        let RegInfo {
            number: vs2,
            bit_length: bit_len2,
            ..
        } = self.expr_regs.get(&left.id).cloned().unwrap();
        let RegInfo {
            number: vs1,
            bit_length: bit_len1,
            ..
        } = self.expr_regs.get(&right.id).cloned().unwrap();
        assert_eq!(bit_len1, bit_len2);
        let vd = if let Some(vd) = exists_vd {
            vd
        } else {
            match op_category {
                OpCategory::Binary | OpCategory::Bool => {
                    let vd = self.v_registers.alloc().ok_or_else(|| {
                        (
                            expr.expr.1,
                            anyhow!("not enough V register for this expression"),
                        )
                    })?;
                    self.expr_regs
                        .insert(expr.id, RegInfo::new(vd, bit_len1, None));
                    vd
                }
                OpCategory::AssignOp => vs2,
            }
        };
        let ivv = Ivv {
            vd: VReg::from_u8(vd),
            vs2: VReg::from_u8(vs2),
            vs1: VReg::from_u8(vs1),
            vm: false,
        };
        let inst = match op {
            // ==== OpCategory::Binary | OpCategory::AssignOp ====
            // The `+` operator (addition)
            // The `+=` operator
            syn::BinOp::Add(_) | syn::BinOp::AddEq(_) => VInst::VaddVv(ivv),
            // The `-` operator (subtraction)
            // The `-=` operator
            syn::BinOp::Sub(_) | syn::BinOp::SubEq(_) => VInst::VsubVv(ivv),
            // The `*` operator (multiplication)
            // The `*=` operator
            syn::BinOp::Mul(_) | syn::BinOp::MulEq(_) => VInst::VmulVv(ivv),
            // The `/` operator (division)
            // The `/=` operator
            syn::BinOp::Div(_) | syn::BinOp::DivEq(_) => VInst::VdivuVv(ivv),
            // The `%` operator (modulus)
            // The `%=` operator
            syn::BinOp::Rem(_) | syn::BinOp::RemEq(_) => VInst::VremuVv(ivv),
            // The `^` operator (bitwise xor)
            // The `^=` operator
            syn::BinOp::BitXor(_) | syn::BinOp::BitXorEq(_) => VInst::VxorVv(ivv),
            // The `&` operator (bitwise and)
            // The `&=` operator
            syn::BinOp::BitAnd(_) | syn::BinOp::BitAndEq(_) => VInst::VandVv(ivv),
            // The `|` operator (bitwise or)
            // The `|=` operator
            syn::BinOp::BitOr(_) | syn::BinOp::BitOrEq(_) => VInst::VorVv(ivv),
            // The `<<` operator (shift left)
            // The `<<=` operator
            syn::BinOp::Shl(_) | syn::BinOp::ShlEq(_) => VInst::VsllVv(ivv),
            // The `>>` operator (shift right)
            // The `>>=` operator
            syn::BinOp::Shr(_) | syn::BinOp::ShrEq(_) => VInst::VsrlVv(ivv),

            // The `&&` operator (logical and)
            // The `||` operator (logical or)
            // NOTE: early returned when check type names
            syn::BinOp::And(_) | syn::BinOp::Or(_) => unreachable!(),

            // ==== OpCategory::Bool ====
            // The `==` operator (equality)
            syn::BinOp::Eq(_) => VInst::VmseqVv(ivv),
            // The `<` operator (less than)
            syn::BinOp::Lt(_) => VInst::VmsltuVv(ivv),
            // The `<=` operator (less than or equal to)
            syn::BinOp::Le(_) => VInst::VmsleuVv(ivv),
            // The `!=` operator (not equal to)
            syn::BinOp::Ne(_) => VInst::VmsneVv(ivv),
            // The `>=` operator (greater than or equal to)
            syn::BinOp::Ge(_) => VInst::VmsgeuVv(ivv),
            // The `>` operator (greater than)
            syn::BinOp::Gt(_) => VInst::VmsgtuVv(ivv),
        };
        if self.show_asm {
            let comment = inst_to_comment(&inst);
            tokens.extend(Some(quote! {
                let _ = #comment;
            }));
        }
        let inst_string = inst_to_string(&inst);
        let ts = quote! {
            unsafe {
                asm!(#inst_string)
            }
        };
        tokens.extend(Some(ts));

        // Handle `Expression::Paren(expr)`, bind current expr register to parent expr.
        if let Some(extra_expr_id) = extra_bind_id {
            if let Some(info) = self.expr_regs.get(&expr.id).cloned() {
                self.expr_regs.insert(extra_expr_id, info);
            }
        }
        self.free_sub_exprs(expr.id, left.id, right.id);

        match op_category {
            OpCategory::Binary if top_level && exists_vd.is_none() => {
                let vreg = {
                    let reg_info = self.expr_regs.get_mut(&expr.id).unwrap();
                    self.v_registers.free(reg_info.number);
                    reg_info.is_freed = true;
                    reg_info.number
                };
                vstore_codegen(&mut tokens, vreg, bit_length, self.show_asm);
                let mut rv = TokenStream::new();
                token::Brace::default().surround(&mut rv, |inner| {
                    inner.extend(Some(tokens));
                });
                Ok(rv)
            }
            OpCategory::Binary | OpCategory::AssignOp => Ok(tokens),
            OpCategory::Bool => {
                let vreg = {
                    let reg_info = self.expr_regs.get_mut(&expr.id).unwrap();
                    self.v_registers.free(reg_info.number);
                    reg_info.is_freed = true;
                    reg_info.number
                };
                let inst = VInst::VfirstM {
                    rd: XReg::T0,
                    vs2: VReg::from_u8(vreg),
                    vm: false,
                };
                if self.show_asm {
                    let comment = inst_to_comment(&inst);
                    tokens.extend(Some(quote! {
                        let _ = #comment;
                        // let _ = "mv {tmp_bool_t0}, t0";
                    }));
                }
                let inst_string = inst_to_string(&inst);
                tokens.extend(Some(quote! {
                    let _tmp_t0_saved: i64;
                    let tmp_bool_t0: i64;
                    // t0: 0  (vms* success)
                    // t0: -1 (not found)
                    unsafe {
                        asm!(
                            // This should be vfirst.m t0, vrs2
                            "mv {0}, t0",
                            #inst_string,
                            "mv {1}, t0",
                            "mv t0, {0}",
                            out (reg) _tmp_t0_saved,
                            out (reg) tmp_bool_t0,
                        )
                    }
                    tmp_bool_t0 == 0
                }));
                let mut rv = TokenStream::new();
                token::Brace::default().surround(&mut rv, |inner| {
                    inner.extend(Some(tokens));
                });
                Ok(rv)
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn gen_method_call_tokens(
        &mut self,
        expr: &TypedExpression,
        receiver: &TypedExpression,
        method: &syn::Ident,
        args: &[TypedExpression],
        top_level: bool,
        extra_bind_id: Option<usize>,
        mut bit_length: u16,
    ) -> Result<TokenStream, SpannedError> {
        let receiver_bit_length: u16 = match receiver.type_name().as_deref() {
            Some("U256") => 256,
            Some("U512") => 512,
            Some("U1024") => 1024,
            _ => self
                .expr_regs
                .get(&expr.id)
                .map(|info| info.bit_length)
                .unwrap_or(0),
        };
        if top_level {
            bit_length = receiver_bit_length;
        }
        if bit_length == 0 {
            return self.default_method_call_codegen(receiver, method, args);
        }

        let method_string = method.to_string();
        let mut tokens = TokenStream::new();
        match method_string.as_str() {
            "wrapping_add" | "wrapping_sub" | "wrapping_mul" | "wrapping_div" | "wrapping_rem"
            | "overflowing_add" | "overflowing_sub" | "overflowing_mul" | "checked_add"
            | "checked_sub" | "checked_mul" | "checked_div" | "checked_rem" | "saturating_add"
            | "saturating_sub" | "saturating_mul" => {
                if args.len() != 1 {
                    return Err((
                        expr.expr.1,
                        anyhow!(
                            "special method call to U256/U512/U1024, must have exact one argument"
                        ),
                    ));
                }
            }
            _ => {
                return self.default_method_call_codegen(receiver, method, args);
            }
        }

        self.update_vconfig(&mut tokens, bit_length);

        let left = &receiver;
        let right = &args[0];
        self.gen_sub_exprs(&mut tokens, left, right, bit_length)?;

        let RegInfo {
            number: vs2,
            bit_length: bit_len2,
            ..
        } = *self.expr_regs.get(&left.id).unwrap();
        let RegInfo {
            number: vs1,
            bit_length: bit_len1,
            ..
        } = *self.expr_regs.get(&right.id).unwrap();
        assert_eq!(bit_len1, bit_len2);
        let vd = self.v_registers.alloc().ok_or_else(|| {
            (
                expr.expr.1,
                anyhow!("not enough V register for this expression"),
            )
        })?;
        self.expr_regs
            .insert(expr.id, RegInfo::new(vd, bit_len1, None));
        // Handle `Expression::Paren(expr)`, bind current expr register to parent expr.
        if let Some(extra_expr_id) = extra_bind_id {
            if let Some(info) = self.expr_regs.get(&expr.id).cloned() {
                self.expr_regs.insert(extra_expr_id, info);
            }
        }

        let ivv = Ivv {
            vd: VReg::from_u8(vd),
            vs2: VReg::from_u8(vs2),
            vs1: VReg::from_u8(vs1),
            vm: false,
        };

        match method_string.as_str() {
            "wrapping_add" => inst_codegen(&mut tokens, VInst::VaddVv(ivv), self.show_asm),
            "wrapping_sub" => inst_codegen(&mut tokens, VInst::VsubVv(ivv), self.show_asm),
            "wrapping_mul" => inst_codegen(&mut tokens, VInst::VmulVv(ivv), self.show_asm),
            "wrapping_div" => inst_codegen(&mut tokens, VInst::VdivuVv(ivv), self.show_asm),
            "wrapping_rem" => inst_codegen(&mut tokens, VInst::VremuVv(ivv), self.show_asm),

            /*
            vadd.vv v1, v2, v3
            vmsltu.vv v4, v1, v2
            vfirst.m t0, v4
            if t0 == 0 {
                (v1, true)
            } else {
                (v1, false)
            }
            */
            "overflowing_add" => {
                self.simple_overflowing_codegen(
                    &mut tokens,
                    VInst::VaddVv(ivv),
                    ivv,
                    bit_length,
                    false,
                )
                .map_err(|err| (expr.expr.1, err))?;
            }
            /*
            vsub.vv v1, v2, v3
            vmsltu.vv v4, v2, v3
            vfirst.m t0, v4
            if t0 == 0 {
                (v1, true)
            } else {
                (v1, false)
            }
             */
            "overflowing_sub" => {
                self.simple_overflowing_codegen(
                    &mut tokens,
                    VInst::VsubVv(ivv),
                    ivv,
                    bit_length,
                    false,
                )
                .map_err(|err| (expr.expr.1, err))?;
            }
            /*
            vmul.vv v1, v2, v3
            vmsne.vi v4 v2, 0
            vfirst.m t0, v4
            if t0 == 0 {
                vdivu.vv v4, v1, v2
                vmsne.vv v4, v4, v3
                vfirst.m t0, v4
                (v1, t0 == 0)
            } else {
                (v1, false)
            }
             */
            "overflowing_mul" => {
                self.overflowing_mul_codegen(&mut tokens, ivv, bit_length, false)
                    .map_err(|err| (expr.expr.1, err))?;
            }

            /*
            let (value, overflow) = self.overflowing_add(other);
            if overflow {
                None
            } else {
                Some(value)
            }
             */
            "checked_add" => {
                self.simple_overflowing_codegen(
                    &mut tokens,
                    VInst::VaddVv(ivv),
                    ivv,
                    bit_length,
                    true,
                )
                .map_err(|err| (expr.expr.1, err))?;
            }

            /*
            vmsltu.vv v4, v2, v3
            vfirst.m t0, v4
            if t0 == 0 {
                None
            } else {
                vsub.vv v1, v2, v3
                Some(v1)
            }
             */
            "checked_sub" => {
                self.checked_sub(&mut tokens, VInst::VsubVv(ivv), ivv, bit_length)
                    .map_err(|err| (expr.expr.1, err))?;
            }

            /*
            vmul.vv v1, v2, v3
            vmsne.vi v4 v2, 0
            vfirst.m t0, v4
            if t0 == 0 {
                vdivu.vv v4, v1, v2
                vmsne.vv v4, v4, v3
                vfirst.m t0, v4
                if t0 == 0 {
                    None
                } else {
                    Some(v1)
                }
            } else {
                Some(v1)
            }
             */
            "checked_mul" => {
                self.overflowing_mul_codegen(&mut tokens, ivv, bit_length, true)
                    .map_err(|err| (expr.expr.1, err))?;
            }

            /*
            vmseq.vi v4, v2, 0  # (v2 == vs1)
            vfirst.m t0, v4
            if t0 == 0 {
                None
            } else {
                vdivu.vv v1, v2, v3
                Some(v1)
            }
             */
            "checked_div" => {
                self.simple_checked_codegen(&mut tokens, VInst::VdivuVv(ivv), ivv, bit_length)
                    .map_err(|err| (expr.expr.1, err))?;
            }
            /*
            vmseq.vi v4, v2, 0  # (v2 == vs1)
            vfirst.m t0, v4
            if t0 == 0 {
                None
            } else {
                vdivu.vv v1, v2, v3
                Some(v1)
            }
             */
            "checked_rem" => {
                self.simple_checked_codegen(&mut tokens, VInst::VremuVv(ivv), ivv, bit_length)
                    .map_err(|err| (expr.expr.1, err))?;
            }

            // vsaddu.vv vd, vs2, vs1, vm
            "saturating_add" => inst_codegen(&mut tokens, VInst::VsadduVv(ivv), self.show_asm),
            // vssubu.vv vd, vs2, vs1, vm
            "saturating_sub" => inst_codegen(&mut tokens, VInst::VssubuVv(ivv), self.show_asm),
            /*
            vmul.vv v1, v2, v3
            vmsne.vi v4 v1, 0
            vfirst.m t0, v4
            if t0 == 0 {
                vdivu.vv v4, v1, v2
                vmsne.vv v4, v4, v3
                vfirst.m t0, v4
                if t0 == 0 {
                    Uxx::max_value()
                } else {
                    v1
                }
            } else {
                v1
            }
             */
            "saturating_mul" => {
                self.saturating_mul_codegen(&mut tokens, ivv, bit_length)
                    .map_err(|err| (expr.expr.1, err))?;
            }
            _ => {}
        };

        self.free_sub_exprs(expr.id, left.id, right.id);

        let is_simple_asm = matches!(
            method_string.as_str(),
            "wrapping_add"
                | "wrapping_sub"
                | "wrapping_mul"
                | "wrapping_div"
                | "wrapping_rem"
                | "saturating_add"
                | "saturating_sub"
        );

        if top_level {
            if is_simple_asm {
                let vreg = {
                    let reg_info = self.expr_regs.get_mut(&expr.id).unwrap();
                    self.v_registers.free(reg_info.number);
                    reg_info.is_freed = true;
                    reg_info.number
                };
                let inst = VInst::VseV {
                    width: bit_length,
                    vs3: VReg::from_u8(vreg),
                    rs1: XReg::T0,
                    vm: false,
                };
                if self.show_asm {
                    let comment1 = inst_to_comment(&inst);
                    tokens.extend(Some(quote! {
                        let _ = #comment1;
                    }));
                }
                let uint_type = quote::format_ident!("U{}", bit_length);
                let buf_length = bit_length as usize / 8;
                let inst_string = inst_to_string(&inst);
                tokens.extend(Some(quote! {
                    let _tmp_t0_saved: i64;
                    let mut tmp_rvv_vector_buf: core::mem::MaybeUninit<[u8; #buf_length]> = core::mem::MaybeUninit::uninit();
                    unsafe {
                        asm!(
                            "mv {0}, t0",
                            "mv t0, {1}",
                            // This should be vse{256, 512, 1024}
                            #inst_string,
                            "mv t0, {0}",
                            out(reg) _tmp_t0_saved,
                            in(reg) tmp_rvv_vector_buf.as_mut_ptr(),
                        )
                    }
                    unsafe { core::mem::transmute::<_, #uint_type>(tmp_rvv_vector_buf) }
                }));
            }
            let mut rv = TokenStream::new();
            token::Brace::default().surround(&mut rv, |inner| {
                inner.extend(Some(tokens));
            });
            Ok(rv)
        } else {
            Ok(tokens)
        }
    }

    fn gen_sub_exprs(
        &mut self,
        tokens: &mut TokenStream,
        left: &TypedExpression,
        right: &TypedExpression,
        bit_length: u16,
    ) -> Result<(), SpannedError> {
        for typed_expr in [left, right] {
            if let Some(var_ident) = typed_expr.expr.0.var_ident() {
                if let Some(vreg) = self.var_regs.get(var_ident) {
                    self.expr_regs.insert(
                        typed_expr.id,
                        RegInfo::new(*vreg, bit_length, Some(var_ident.clone())),
                    );
                } else {
                    // Load{256,512,1024}
                    let vreg = self.v_registers.alloc().ok_or_else(|| {
                        (
                            typed_expr.expr.1,
                            anyhow!("not enough V register for this expression"),
                        )
                    })?;
                    let inst = VInst::VleV {
                        width: bit_length,
                        vd: VReg::from_u8(vreg),
                        rs1: XReg::T0,
                        vm: false,
                    };
                    if self.show_asm {
                        let comment = inst_to_comment(&inst);
                        tokens.extend(Some(quote! {
                            let _ = #comment;
                        }));
                    }
                    let inst_string = inst_to_string(&inst);
                    let ts = quote! {
                        let _tmp_t0_saved: i64;
                        unsafe {
                            asm!(
                                "mv {0}, t0",
                                "mv t0, {1}",
                                #inst_string,
                                "mv t0, {0}",
                                out(reg) _tmp_t0_saved,
                                in(reg) #var_ident.as_ref().as_ptr(),
                            )
                        }
                    };
                    tokens.extend(Some(ts));
                    self.var_regs.insert(var_ident.clone(), vreg);
                    self.expr_regs.insert(
                        typed_expr.id,
                        RegInfo::new(vreg, bit_length, Some(var_ident.clone())),
                    );
                }
            } else {
                let ts = self.gen_tokens(typed_expr, false, None, None, bit_length)?;
                tokens.extend(Some(ts));
            }
        }
        Ok(())
    }

    pub(crate) fn gen_inputs_tokens(
        &mut self,
        tokens: &mut TokenStream,
    ) -> Result<(), SpannedError> {
        if let Some(fn_args) = self.fn_args.take() {
            let mut args = fn_args
                .into_iter()
                .filter(|fn_arg| {
                    self.variables
                        .get(&fn_arg.name)
                        .map(|info| !info.is_unused())
                        .expect("function input variable")
                })
                .filter_map(|fn_arg| {
                    let bit_length: u16 = match fn_arg.ty.0.type_name().as_deref() {
                        Some("U256") => 256,
                        Some("U512") => 512,
                        Some("U1024") => 1024,
                        _ => 0,
                    };
                    if bit_length > 0 {
                        Some((fn_arg, bit_length))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            args.sort_by_key(|(_, bit_length)| *bit_length);

            for (fn_arg, bit_length) in args {
                self.update_vconfig(tokens, bit_length);
                // Load{256,512,1024}
                let vreg = match self.v_registers.alloc() {
                    Some(vreg) => vreg,
                    None => {
                        return Err((
                            fn_arg.span,
                            anyhow!("not enough V register for function argument"),
                        ));
                    }
                };
                let inst = VInst::VleV {
                    width: bit_length,
                    vd: VReg::from_u8(vreg),
                    rs1: XReg::T0,
                    vm: false,
                };
                if self.show_asm {
                    let comment = inst_to_comment(&inst);
                    tokens.extend(Some(quote! {
                        let _ = #comment;
                    }));
                }
                let var_ident = fn_arg.name;
                let inst_string = inst_to_string(&inst);
                let ts = quote! {
                    let _tmp_t0_saved: i64;
                    unsafe {
                        asm!(
                            "mv {0}, t0",
                            "mv t0, {1}",
                            #inst_string,
                            "mv t0, {0}",
                            out(reg) _tmp_t0_saved,
                            in(reg) #var_ident.as_ref().as_ptr(),
                        )
                    }
                };
                tokens.extend(Some(ts));
                self.var_regs.insert(var_ident.clone(), vreg);
            }
        }
        Ok(())
    }

    fn update_vconfig(&mut self, tokens: &mut TokenStream, bit_length: u16) {
        // vsetvli x0, t0, e{256,512,1024}, m1, ta, ma
        let v_config = VConfig::Vsetvli {
            rd: XReg::Zero,
            rs1: XReg::T0,
            vtypei: Vtypei::new(bit_length, Vlmul::M1, true, true),
        };
        if self.v_config.as_ref() != Some(&v_config) {
            self.v_config = Some(v_config);
            let inst = VInst::VConfig(v_config);
            if self.show_asm {
                let comment1 = inst_to_comment(&inst);
                tokens.extend(Some(quote! {
                    let _ = #comment1;
                }));
            }
            let inst_string = inst_to_string(&inst);
            let ts = quote! {
                unsafe {
                    asm!(
                        "li t0, 1",  // AVL = 1
                        #inst_string,
                    )
                }
            };
            tokens.extend(Some(ts));
        }
    }

    fn free_sub_exprs(
        &mut self,
        current_expr_id: usize,
        left_expr_id: usize,
        right_expr_id: usize,
    ) {
        for sub_expr_id in [&left_expr_id, &right_expr_id] {
            let sub_reg_info = self.expr_regs.get_mut(sub_expr_id).unwrap();
            let should_free = if let Some(var_ident) = sub_reg_info.var_ident.as_ref() {
                let var_info = self.variables.get(var_ident).unwrap();
                var_info.end_expr_id <= current_expr_id
            } else {
                true
            };
            if should_free {
                if !sub_reg_info.is_freed {
                    self.v_registers.free(sub_reg_info.number);
                    sub_reg_info.is_freed = true;
                } else {
                    panic!("double free expression: {}", sub_expr_id);
                }
            }
        }
    }

    fn simple_checked_codegen(
        &mut self,
        tokens: &mut TokenStream,
        inst: VInst,
        ivv: Ivv,
        bit_length: u16,
    ) -> Result<(), anyhow::Error> {
        let eq_vd = self
            .v_registers
            .alloc()
            .ok_or_else(|| anyhow!("not enough V register for this expression"))?;
        let mut inner_tokens = TokenStream::new();
        let inst_mseq = VInst::VmseqVi(Ivi {
            vd: VReg::from_u8(eq_vd),
            vs2: ivv.vs1,
            imm: Imm(0),
            vm: false,
        });
        let inst_firstm = VInst::VfirstM {
            rd: XReg::T0,
            vs2: VReg::from_u8(eq_vd),
            vm: false,
        };
        let inst_store = VInst::VseV {
            width: bit_length,
            vs3: ivv.vd,
            rs1: XReg::T0,
            vm: false,
        };
        let uint_type = quote::format_ident!("U{}", bit_length);
        let buf_length = bit_length as usize / 8;

        if self.show_asm {
            let comment0 = inst_to_comment(&inst_mseq);
            let comment1 = inst_to_comment(&inst_firstm);
            let comment2 = inst_to_comment(&inst);
            let comment3 = inst_to_comment(&inst_store);
            inner_tokens.extend(Some(quote! {
                let _ = #comment0;
                let _ = #comment1;
                let _ = #comment2;
                let _ = #comment3;
            }));
        }
        let inst_mseq_string = inst_to_string(&inst_mseq);
        let inst_firstm_string = inst_to_string(&inst_firstm);
        let inst_string = inst_to_string(&inst);
        let inst_store_string = inst_to_string(&inst_store);
        let ts = quote! {
            let mut _tmp_t0_saved: i64;
            let tmp_bool_t0: i64;
            // tn: 0  (vms* success)
            // tn: -1 (not found)
            unsafe {
                asm!(
                    "mv {0}, t0",
                    #inst_mseq_string,
                    #inst_firstm_string,
                    "mv {1}, t0",
                    "mv t0, {0}",
                    out(reg) _tmp_t0_saved,
                    out(reg) tmp_bool_t0,
                )
            }
            if tmp_bool_t0 == 0 {
                None
            } else {
                let mut tmp_rvv_vector_buf: core::mem::MaybeUninit<[u8; #buf_length]> = core::mem::MaybeUninit::uninit();
                unsafe {
                    asm!(
                        "mv {0}, t0",
                        #inst_string,
                        "mv t0, {1}",
                        #inst_store_string,
                        "mv t0, {0}",
                        out(reg) _tmp_t0_saved,
                        in(reg) tmp_rvv_vector_buf.as_mut_ptr(),
                    )
                }
                Some(unsafe { core::mem::transmute::<_, #uint_type>(tmp_rvv_vector_buf) })
            }
        };
        inner_tokens.extend(Some(ts));
        token::Brace::default().surround(tokens, |inner| {
            inner.extend(Some(inner_tokens));
        });
        Ok(())
    }

    // TODO: merge this method with simple_checked_codegen()
    fn checked_sub(
        &mut self,
        tokens: &mut TokenStream,
        inst: VInst,
        ivv: Ivv,
        bit_length: u16,
    ) -> Result<(), anyhow::Error> {
        let lt_vd = self
            .v_registers
            .alloc()
            .ok_or_else(|| anyhow!("not enough V register for this expression"))?;
        let mut inner_tokens = TokenStream::new();

        let inst_msltu = VInst::VmsltuVv(Ivv {
            vd: VReg::from_u8(lt_vd),
            vs2: ivv.vs2,
            vs1: ivv.vs1,
            vm: false,
        });
        let inst_firstm = VInst::VfirstM {
            rd: XReg::T0,
            vs2: VReg::from_u8(lt_vd),
            vm: false,
        };
        let inst_store = VInst::VseV {
            width: bit_length,
            vs3: ivv.vd,
            rs1: XReg::T0,
            vm: false,
        };
        let uint_type = quote::format_ident!("U{}", bit_length);
        let buf_length = bit_length as usize / 8;

        if self.show_asm {
            let comment0 = inst_to_comment(&inst_msltu);
            let comment1 = inst_to_comment(&inst_firstm);
            let comment2 = inst_to_comment(&inst);
            let comment3 = inst_to_comment(&inst_store);
            inner_tokens.extend(Some(quote! {
                let _ = #comment0;
                let _ = #comment1;
                let _ = #comment2;
                let _ = #comment3;
            }));
        }
        let inst_msltu_string = inst_to_string(&inst_msltu);
        let inst_firstm_string = inst_to_string(&inst_firstm);
        let inst_string = inst_to_string(&inst);
        let inst_store_string = inst_to_string(&inst_store);
        let ts = quote! {
            let mut _tmp_t0_saved: i64;
            let tmp_bool_t0: i64;
            // tn: 0  (vms* success)
            // tn: -1 (not found)
            unsafe {
                asm!(
                    "mv {0}, t0",
                    #inst_msltu_string,
                    #inst_firstm_string,
                    "mv {1}, t0",
                    "mv t0, {0}",
                    out(reg) _tmp_t0_saved,
                    out(reg) tmp_bool_t0,
                )
            }
            if tmp_bool_t0 == 0 {
                None
            } else {
                let mut tmp_rvv_vector_buf: core::mem::MaybeUninit<[u8; #buf_length]> = core::mem::MaybeUninit::uninit();
                unsafe {
                    asm!(
                        #inst_string,
                        "mv {0}, t0",
                        "mv t0, {1}",
                        #inst_store_string,
                        "mv t0, {0}",
                        out(reg) _tmp_t0_saved,
                        in(reg) tmp_rvv_vector_buf.as_mut_ptr(),
                    )
                }
                Some(unsafe { core::mem::transmute::<_, #uint_type>(tmp_rvv_vector_buf) })
            }
        };
        inner_tokens.extend(Some(ts));
        token::Brace::default().surround(tokens, |inner| {
            inner.extend(Some(inner_tokens));
        });
        Ok(())
    }

    fn simple_overflowing_codegen(
        &mut self,
        tokens: &mut TokenStream,
        inst: VInst,
        ivv: Ivv,
        bit_length: u16,
        // checked_{add,sub}()
        is_checked: bool,
    ) -> Result<(), anyhow::Error> {
        let lt_vd = self
            .v_registers
            .alloc()
            .ok_or_else(|| anyhow!("not enough V register for this expression"))?;
        let mut inner_tokens = TokenStream::new();
        if is_checked {
            checked_rv_codegen(
                &mut inner_tokens,
                inst,
                ivv,
                lt_vd,
                bit_length,
                self.show_asm,
            );
        } else {
            overflowing_rv_codegen(
                &mut inner_tokens,
                inst,
                ivv,
                lt_vd,
                bit_length,
                self.show_asm,
            );
        }
        token::Brace::default().surround(tokens, |inner| {
            inner.extend(Some(inner_tokens));
        });
        Ok(())
    }

    fn overflowing_mul_codegen(
        &mut self,
        tokens: &mut TokenStream,
        ivv: Ivv,
        bit_length: u16,
        // is checked_mul()
        is_checked: bool,
    ) -> Result<(), anyhow::Error> {
        let vd = self
            .v_registers
            .alloc()
            .ok_or_else(|| anyhow!("not enough V register for this expression"))?;
        let mut inner_tokens = TokenStream::new();

        let inst_mul = VInst::VmulVv(ivv);
        let inst_msne = VInst::VmsneVi(Ivi {
            vd: VReg::from_u8(vd),
            vs2: ivv.vd,
            imm: Imm(0),
            vm: false,
        });
        let inst_firstm1 = VInst::VfirstM {
            rd: XReg::T0,
            vs2: VReg::from_u8(vd),
            vm: false,
        };
        let inst_div = VInst::VdivuVv(Ivv {
            vd: VReg::from_u8(vd),
            vs2: ivv.vd,
            vs1: ivv.vs2,
            vm: false,
        });
        let inst_ne = VInst::VmsneVv(Ivv {
            vd: VReg::from_u8(vd),
            vs2: VReg::from_u8(vd),
            vs1: ivv.vs1,
            vm: false,
        });
        let inst_firstm2 = VInst::VfirstM {
            rd: XReg::T0,
            vs2: VReg::from_u8(vd),
            vm: false,
        };
        let inst_store = VInst::VseV {
            width: bit_length,
            vs3: ivv.vd,
            rs1: XReg::T0,
            vm: false,
        };

        let uint_type = quote::format_ident!("U{}", bit_length);
        let buf_length = bit_length as usize / 8;
        if self.show_asm {
            let comment0 = inst_to_comment(&inst_mul);
            let comment1 = inst_to_comment(&inst_msne);
            let comment2 = inst_to_comment(&inst_firstm1);
            let comment3 = inst_to_comment(&inst_store);
            inner_tokens.extend(Some(quote! {
                let _ = #comment0;
                let _ = #comment1;
                let _ = #comment2;
                let _ = #comment3;
            }));
        }
        let inst_mul_string = inst_to_string(&inst_mul);
        let inst_msne_string = inst_to_string(&inst_msne);
        let inst_firstm1_string = inst_to_string(&inst_firstm1);
        let inst_div_string = inst_to_string(&inst_div);
        let inst_ne_string = inst_to_string(&inst_ne);
        let inst_firstm2_string = inst_to_string(&inst_firstm2);
        let inst_store_string = inst_to_string(&inst_store);
        inner_tokens.extend(Some(quote! {
            let mut _tmp_t0_saved: i64;
            let mut tmp_bool_t0: i64;
            let mut tmp_rvv_vector_buf: core::mem::MaybeUninit<[u8; #buf_length]> = core::mem::MaybeUninit::uninit();
            unsafe {
                asm!(
                    "mv {0}, t0",
                    // vmul.vv v1, v2, v3
                    #inst_mul_string,
                    // vmsne.vi v4 v2, 0
                    #inst_msne_string,
                    // vfirst.m t0, v4
                    #inst_firstm1_string,
                    "mv {1}, t0",
                    "mv t0, {2}",
                    // vse{n}.v v1, (t0)
                    #inst_store_string,
                    "mv t0, {0}",
                    out(reg) _tmp_t0_saved,
                    out(reg) tmp_bool_t0,
                    in(reg) tmp_rvv_vector_buf.as_mut_ptr(),
                )
            }
            let tmp_uint_rv = unsafe { core::mem::transmute::<_, #uint_type>(tmp_rvv_vector_buf) };
        }));
        if self.show_asm {
            let comment0 = inst_to_comment(&inst_div);
            let comment1 = inst_to_comment(&inst_ne);
            let comment2 = inst_to_comment(&inst_firstm2);
            inner_tokens.extend(Some(quote! {
                let _ = #comment0;
                let _ = #comment1;
                let _ = #comment2;
            }));
        }
        let ts = if is_checked {
            quote! {
                if tmp_bool_t0 == 0 {
                    unsafe {
                        asm!(
                            "mv {0}, t0",
                            // vdivu.vv v4, v1, v2
                            #inst_div_string,
                            // vmsne.vv v4, v4, v3
                            #inst_ne_string,
                            // vfirst.m t0, v4
                            #inst_firstm2_string,
                            "mv {1}, t0",
                            "mv t0, {0}",
                            out(reg) _tmp_t0_saved,
                            out(reg) tmp_bool_t0,
                        )
                    }
                    if tmp_bool_t0 == 0 {
                        None
                    } else {
                        Some(tmp_uint_rv)
                    }
                } else {
                    Some(tmp_uint_rv)
                }
            }
        } else {
            quote! {
                if tmp_bool_t0 == 0 {
                    unsafe {
                        asm!(
                            "mv {0}, t0",
                            // vdivu.vv v5, v1, v2
                            #inst_div_string,
                            // vmsne.vv v4, v5, v3
                            #inst_ne_string,
                            // vfirst.m t0, v4
                            #inst_firstm2_string,
                            "mv {1}, t0",
                            "mv t0, {0}",
                            out(reg) _tmp_t0_saved,
                            out(reg) tmp_bool_t0,
                        )
                    }
                    (tmp_uint_rv, tmp_bool_t0 == 0)
                } else {
                    (tmp_uint_rv, false)
                }
            }
        };
        inner_tokens.extend(Some(ts));
        token::Brace::default().surround(tokens, |inner| {
            inner.extend(Some(inner_tokens));
        });
        Ok(())
    }

    fn saturating_mul_codegen(
        &mut self,
        tokens: &mut TokenStream,
        ivv: Ivv,
        bit_length: u16,
    ) -> Result<(), anyhow::Error> {
        let vd = self
            .v_registers
            .alloc()
            .ok_or_else(|| anyhow!("not enough V register for this expression"))?;

        let mut inner_tokens = TokenStream::new();
        let inst_mul = VInst::VmulVv(ivv);
        let inst_msne = VInst::VmsneVi(Ivi {
            vd: VReg::from_u8(vd),
            vs2: ivv.vd,
            imm: Imm(0),
            vm: false,
        });
        let inst_firstm1 = VInst::VfirstM {
            rd: XReg::T0,
            vs2: VReg::from_u8(vd),
            vm: false,
        };
        let inst_div = VInst::VdivuVv(Ivv {
            vd: VReg::from_u8(vd),
            vs2: ivv.vd,
            vs1: ivv.vs2,
            vm: false,
        });
        let inst_ne = VInst::VmsneVv(Ivv {
            vd: VReg::from_u8(vd),
            vs2: VReg::from_u8(vd),
            vs1: ivv.vs1,
            vm: false,
        });
        let inst_firstm2 = VInst::VfirstM {
            rd: XReg::T0,
            vs2: VReg::from_u8(vd),
            vm: false,
        };
        let inst_store = VInst::VseV {
            width: bit_length,
            vs3: ivv.vd,
            rs1: XReg::T0,
            vm: false,
        };
        let uint_type = quote::format_ident!("U{}", bit_length);
        let buf_length = bit_length as usize / 8;
        let inst_mul_string = inst_to_string(&inst_mul);
        let inst_msne_string = inst_to_string(&inst_msne);
        let inst_firstm1_string = inst_to_string(&inst_firstm1);
        let inst_div_string = inst_to_string(&inst_div);
        let inst_ne_string = inst_to_string(&inst_ne);
        let inst_firstm2_string = inst_to_string(&inst_firstm2);
        let inst_store_string = inst_to_string(&inst_store);
        inner_tokens.extend(Some(quote! {
            let mut _tmp_t0_saved: i64;
            let mut tmp_bool_t0: i64;
            unsafe {
                asm!(
                    "mv {0}, t0",
                    #inst_mul_string,
                    #inst_msne_string,
                    #inst_firstm1_string,
                    "mv {1}, t0",
                    "mv t0, {0}",
                    out(reg) _tmp_t0_saved,
                    out(reg) tmp_bool_t0,
                )
            }
            if tmp_bool_t0 == 0 {
                unsafe {
                    asm!(
                        "mv {0}, t0",
                        #inst_div_string,
                        #inst_ne_string,
                        #inst_firstm2_string,
                        "mv {1}, t0",
                        "mv t0, {0}",
                        out(reg) _tmp_t0_saved,
                        out(reg) tmp_bool_t0,
                    )
                }
            }
            if tmp_bool_t0 == 0 {
                #uint_type::max_value()
            } else {
                let mut tmp_rvv_vector_buf: core::mem::MaybeUninit<[u8; #buf_length]> = core::mem::MaybeUninit::uninit();
                unsafe {
                    asm!(
                        "mv {0}, t0",
                        "mv t0, {1}",
                        #inst_store_string,
                        "mv t0, {0}",
                        out(reg) _tmp_t0_saved,
                        in(reg) tmp_rvv_vector_buf.as_mut_ptr(),
                    )
                }
                unsafe { core::mem::transmute::<_, #uint_type>(tmp_rvv_vector_buf) }
            }
        }));
        token::Brace::default().surround(tokens, |inner| {
            inner.extend(Some(inner_tokens));
        });
        Ok(())
    }
}

fn vstore_codegen(tokens: &mut TokenStream, vreg: u8, bit_length: u16, show_asm: bool) {
    let inst = VInst::VseV {
        width: bit_length,
        vs3: VReg::from_u8(vreg),
        rs1: XReg::T0,
        vm: false,
    };
    if show_asm {
        let comment = inst_to_comment(&inst);
        tokens.extend(Some(quote! {
            let _ = #comment;
        }));
    }
    let uint_type = quote::format_ident!("U{}", bit_length);
    let buf_length = bit_length as usize / 8;
    let inst_string = inst_to_string(&inst);
    tokens.extend(Some(quote! {
        let _tmp_t0_saved: i64;
        let mut tmp_rvv_vector_buf: core::mem::MaybeUninit<[u8; #buf_length]> = core::mem::MaybeUninit::uninit();
        unsafe {
            asm!(
                "mv {0}, t0",
                "mv t0, {1}",
                // This should be vse{256, 512, 1024}
                #inst_string,
                "mv t0, {0}",
                out(reg) _tmp_t0_saved,
                in(reg) tmp_rvv_vector_buf.as_mut_ptr(),
            )
        }
        unsafe { core::mem::transmute::<_, #uint_type>(tmp_rvv_vector_buf) }
    }));
}

fn inst_codegen(tokens: &mut TokenStream, inst: VInst, show_asm: bool) {
    if show_asm {
        let comment = inst_to_comment(&inst);
        tokens.extend(Some(quote! {
            let _ = #comment;
        }));
    }
    let inst_string = inst_to_string(&inst);
    let ts = quote! {
        unsafe {
            asm!(#inst_string)
        }
    };
    tokens.extend(Some(ts));
}

fn overflowing_rv_codegen(
    tokens: &mut TokenStream,
    inst: VInst,
    ivv: Ivv,
    lt_vd: u8,
    bit_length: u16,
    show_asm: bool,
) {
    let inst_msltu = VInst::VmsltuVv(Ivv {
        vd: VReg::from_u8(lt_vd),
        vs2: ivv.vd,
        vs1: ivv.vs1,
        vm: false,
    });
    let inst_firstm = VInst::VfirstM {
        rd: XReg::T0,
        vs2: VReg::from_u8(lt_vd),
        vm: false,
    };
    let inst_store = VInst::VseV {
        width: bit_length,
        vs3: ivv.vd,
        rs1: XReg::T0,
        vm: false,
    };
    if show_asm {
        let comment0 = inst_to_comment(&inst);
        let comment1 = inst_to_comment(&inst_msltu);
        let comment2 = inst_to_comment(&inst_firstm);
        let comment3 = inst_to_comment(&inst_store);
        tokens.extend(Some(quote! {
            let _ = #comment0;
            let _ = #comment1;
            let _ = #comment2;
            let _ = #comment3;
        }));
    }

    let uint_type = quote::format_ident!("U{}", bit_length);
    let buf_length = bit_length as usize / 8;
    let inst_string = inst_to_string(&inst);
    let inst_msltu_string = inst_to_string(&inst_msltu);
    let inst_firstm_string = inst_to_string(&inst_firstm);
    let inst_store_string = inst_to_string(&inst_store);
    tokens.extend(Some(quote! {
        let _tmp_t0_saved: i64;
        let tmp_bool_t0: i64;
        let mut tmp_rvv_vector_buf: core::mem::MaybeUninit<[u8; #buf_length]> = core::mem::MaybeUninit::uninit();
        // t0: 0  (vms* success)
        // t0: -1 (not found)
        unsafe {
            asm!(
                "mv {0}, t0",
                #inst_string,
                #inst_msltu_string,
                #inst_firstm_string,
                "mv {1}, t0",
                "mv t0, {2}",
                #inst_store_string,
                "mv t0, {0}",
                out(reg) _tmp_t0_saved,
                out(reg) tmp_bool_t0,
                in(reg) tmp_rvv_vector_buf.as_mut_ptr(),
            )
        }
        (
            unsafe { core::mem::transmute::<_, #uint_type>(tmp_rvv_vector_buf) },
            tmp_bool_t0 == 0
        )
    }));
}

fn checked_rv_codegen(
    tokens: &mut TokenStream,
    inst: VInst,
    ivv: Ivv,
    lt_vd: u8,
    bit_length: u16,
    show_asm: bool,
) {
    let inst_msltu = VInst::VmsltuVv(Ivv {
        vd: VReg::from_u8(lt_vd),
        vs2: ivv.vd,
        vs1: ivv.vs1,
        vm: false,
    });
    let inst_firstm = VInst::VfirstM {
        rd: XReg::T0,
        vs2: VReg::from_u8(lt_vd),
        vm: false,
    };
    let inst_store = VInst::VseV {
        width: bit_length,
        vs3: ivv.vd,
        rs1: XReg::T0,
        vm: false,
    };
    if show_asm {
        let comment0 = inst_to_comment(&inst);
        let comment1 = inst_to_comment(&inst_msltu);
        let comment2 = inst_to_comment(&inst_firstm);
        let comment3 = inst_to_comment(&inst_store);
        tokens.extend(Some(quote! {
            let _ = #comment0;
            let _ = #comment1;
            let _ = #comment2;
            let _ = #comment3;
        }));
    }

    let uint_type = quote::format_ident!("U{}", bit_length);
    let buf_length = bit_length as usize / 8;
    let inst_string = inst_to_string(&inst);
    let inst_msltu_string = inst_to_string(&inst_msltu);
    let inst_firstm_string = inst_to_string(&inst_firstm);
    let inst_store_string = inst_to_string(&inst_store);
    tokens.extend(Some(quote! {
        let mut _tmp_t0_saved: i64;
        let tmp_bool_t0: i64;
        // t0: 0  (vms* success)
        // t0: -1 (not found)
        unsafe {
            asm!(
                "mv {0}, t0",
                #inst_string,
                #inst_msltu_string,
                #inst_firstm_string,
                "mv {1}, t0",
                "mv t0, {0}",
                out(reg) _tmp_t0_saved,
                out(reg) tmp_bool_t0,
            )
        }
        if tmp_bool_t0 == 0 {
            None
        } else {
            let mut tmp_rvv_vector_buf: core::mem::MaybeUninit<[u8; #buf_length]> = core::mem::MaybeUninit::uninit();
            unsafe {
                asm!(
                    "mv {0}, t0",
                    "mv t0, {1}",
                    #inst_store_string,
                    "mv t0, {0}",
                    out(reg) _tmp_t0_saved,
                    in(reg) tmp_rvv_vector_buf.as_mut_ptr(),
                )
            }
            Some(unsafe { core::mem::transmute::<_, #uint_type>(tmp_rvv_vector_buf) })
        }
    }));
}

fn inst_to_string(inst: &VInst) -> String {
    let [b0, b1, b2, b3] = inst.encode_bytes();
    format!(".byte {:#04x}, {:#04x}, {:#04x}, {:#04x}", b0, b1, b2, b3)
}
fn inst_to_comment(inst: &VInst) -> String {
    format!("{} - {}", inst, inst.encode_u32())
}
