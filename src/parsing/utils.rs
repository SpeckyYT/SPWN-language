macro_rules! operators {

    (
        Assign: [$($a_tok:ident),+];
        $(
            $( Left => [$($l_tok:ident),+] )?
            $( Right => [$($r_tok:ident),+] )?
            $( Unary => [$($u_tok:ident),+] )?
            ;
        )+
    ) => {
        pub mod operators {
            use crate::lexing::tokens::Token;

            #[derive(Debug, Clone, Copy)]
            pub enum AssignOp {
                $(
                    $a_tok,
                )+
            }
            pub fn is_assign_op(op: Token) -> bool {
                match op {
                    $(
                        Token::$a_tok => true,
                    )+
                    _ => false,
                }
            }


            #[derive(Debug, Copy, Clone, PartialEq, Eq)]
            pub enum OpType {
                Left,
                Right,
                Unary,
            }

            #[derive(Debug, Clone, Copy)]
            pub enum BinOp {
                $(
                    $($($l_tok,)+)?
                    $($($r_tok,)+)?
                )+
            }
            #[derive(Debug, Clone, Copy)]
            pub enum UnaryOp {
                $(
                    $($($u_tok,)+)?
                )+
            }

            impl BinOp {
                pub fn to_str(self) -> &'static str {
                    match self {
                        $(
                            $($(
                                BinOp::$l_tok => Token::$l_tok.to_str(),
                            )+)?
                            $($(
                                BinOp::$r_tok => Token::$r_tok.to_str(),
                            )+)?
                        )+
                    }
                }
            }
            impl UnaryOp {
                pub fn to_str(self) -> &'static str {
                    match self {
                        $(
                            $($(
                                UnaryOp::$u_tok => Token::$u_tok.to_str(),
                            )+)?
                        )+
                    }
                }
            }

            impl Token {
                pub fn to_bin_op(self) -> BinOp {
                    match self {
                        $(
                            $($(Token::$l_tok => BinOp::$l_tok,)+)?
                            $($(Token::$r_tok => BinOp::$r_tok,)+)?
                        )+
                        _ => unreachable!(),
                    }
                }
                pub fn to_unary_op(self) -> UnaryOp {
                    match self {
                        $(
                            $($(Token::$u_tok => UnaryOp::$u_tok,)+)?
                        )+
                        _ => unreachable!(),
                    }
                }
                pub fn to_assign_op(self) -> AssignOp {
                    match self {
                        $(
                            Token::$a_tok => AssignOp::$a_tok,
                        )+
                        _ => unreachable!(),
                    }
                }
            }

            const OP_LIST: &[(OpType, &[Token])] = &[
                $(
                    $( (OpType::Left, &[$(Token::$l_tok),*]) )?
                    $( (OpType::Right, &[$(Token::$r_tok),*]) )?
                    $( (OpType::Unary, &[$(Token::$u_tok),*]) )?
                ),*
            ];
            pub const OP_COUNT: usize = OP_LIST.len();

            pub fn next_infix(prec: usize) -> Option<usize> {
                let mut next = prec + 1;
                while next < OP_COUNT {
                    if OP_LIST[next].0 != OpType::Unary {
                        return Some(next);
                    } else {
                        next += 1;
                    }
                }
                None
            }
            pub fn is_infix_prec(op: Token, prec: usize) -> bool {
                for (i, (typ, toks)) in OP_LIST.iter().enumerate() {
                    if *typ != OpType::Unary && toks.contains(&op) && i == prec {
                        return true
                    }
                }
                return false
            }
            pub fn unary_prec(op: Token) -> Option<usize> {
                for (i, (typ, toks)) in OP_LIST.iter().enumerate() {
                    if *typ == OpType::Unary && toks.contains(&op) {
                        return Some(i)
                    }
                }
                return None
            }
            pub fn prec_type(prec: usize) -> OpType {
                OP_LIST[prec].0
            }
        }
    };
}

operators! {
    Assign: [Assign, PlusEq, MinusEq, MultEq, DivEq, PowEq, ModEq, BinAndEq, BinOrEq, BinNotEq, ShiftLeftEq, ShiftRightEq];
    // lowest precedence
    // Right => [Assign];
    // Right => [PlusEq, MinusEq, MultEq, DivEq, PowEq, ModEq, BinAndEq, BinOrEq, BinNotEq, ShiftLeftEq, ShiftRightEq];
    Left => [Range];
    Left => [In];
    Left => [Is];
    Left => [BinOr, Or];
    Left => [BinAnd, And];
    Unary => [BinNot];
    Unary => [Eq, Neq, Gt, Gte, Lt, Lte];
    Unary => [ExclMark];
    Left => [Eq, Neq, Gt, Gte, Lt, Lte];
    Left => [ShiftLeft, ShiftRight];
    Left => [Plus, Minus];
    Unary => [Minus];
    Left => [Mult, Div, Mod];
    Right => [Pow];
    Left => [As];
    // highest precedence
}
