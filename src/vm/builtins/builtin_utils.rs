/*
fn poo(v: Vec<ValueKey>, vm: &mut Vm, area: CodeArea) -> RuntimeResult<Value> {
    mod arg1 {
        pub struct Arg1 {
            group: Id,
            sfddsfsf: Id
        };
    }
    use arg1::Arg1;

    mod arg2 {
        pub struct Arg2(i64);
    }
    use arg2::Arg2;

    mod arg4 {
        pub struct StringGetter(ValueKey);

        pub struct StringRef<'a>(&'a String);
        pub struct RangeRef<'a>(&'a (i64,));
        pub struct StringMutRef<'a>(&'a mut String);

        impl StringGetter {
            pub fn get_ref(&self, vm: &Vm) -> StringRef<'_> {
                match &vm.memory[self.0].value {
                    Value::String(s) => StringRef(s)
                    _ => panic!("valuekey does not point to value of correct type !!!!!!!!")
                }
            }
            pub fn get_mut_ref(&self, vm: &mut Vm) -> StringMutRef<'_> {
                match &vm.memory[self.0].value {
                    Value::String(s) => StringMutRef(s)
                    _ => panic!("valuekey does not point to value of correct type !!!!!!!!")
                }
            }
        }

        pub enum Arg4 {
            String(StringGetter),
            Float(f64),
            TriggerFunction {
                a: id,
                b: id
            }
        }
    }
    use arg4::Arg4;

    let Value::String(s) = vm.memory[v[0]].value.clone() else {
        unreachable!();
    };
    let arg1 = match vm.memory[v[0]].value {
        Value::TriggerFunction {
            group, dfsfsdf
        } => Arg1 { group, dfsfsfsfsgr},
        ...
    }

}

match arg4.get() {
    AFloat(f) =>
}

*/

// spwn_codegen::def_type! {
//     /// aaa
//     #[raw( #[deprecated] )]
//     impl @string {
//         /// bbb
//         const A = Range(0, 0, 0);

//         fn poo(
//             //String(s) as self = r#"obj { HSV: "aaa",  }"#,
//             //arg1: Int | Int = 10,
//             //arg2: &Int,
//             //Range(start, end, step) as arg2 where Key(b_k),
//             arg4: &String | Float,
//         ) {
//             // block
//         }

//         // fn poo() {}

//         // fn poo(&self) {}

//         // /// ccc
//         // fn poo(&self) -> Test {}

//         // fn poo(
//         //     &self,
//         //     Thing1 as r,
//         //     Thing2 { a, b } as r,
//         //     Thing3(a, b) as r where Key(k),
//         //     a: A | B,
//         //     b: &C,
//         //     c: &D,
//         //     d: &E | &F |, // enum D { E(ERef), F(FRef) } .get_ref
//         //     ...e,
//         //     f where Key(k),
//         //     g where Area(a) Key(k),
//         // ) -> Test {}
//     }
// }

// #[rustfmt::skip]
macro_rules! impl_type {
    (
        $(#[doc = $impl_doc:literal])*
        // temporary until 1.0
        $(#[raw($($impl_raw:tt)*)])?
        impl $impl_var:ident {
            Constants:
            $(
                $(#[doc = $const_doc:literal])*
                // temporary until 1.0
                $(#[raw($($const_raw:tt)*)])?
                const $const:ident = $c_name:ident
                                        $( ( $( $c_val:expr ),* ) )?
                                        $( { $( $c_n:ident: $c_val_s:expr ,)* } )?;
            )*

            Functions:
            $(
                $(#[doc = $fn_doc:literal])*
                // temporary until 1.0
                $(#[raw($($fn_raw:tt)*)])?
                fn $fn_name:ident($(
                    $arg_name:ident
                        $(:
                            $(
                                $(&$ref_ty:ident)? $($deref_ty:ident)?
                            )|+
                        )?
                        $(
                            $(
                                ( $( $v_val:ident ),* )
                            )?
                            $(
                                { $( $v_n:ident $(: $v_val_s:ident)? ,)* }
                            )?
                            as $binder:ident
                        )?

                    $(
                        = $default:literal
                    )?

                    $(
                        where $($extra:ident($extra_bind:ident))+
                    )?

                    ,
                )*) $(-> $ret_type:ident)? $b:block
            )*
        }
    ) => {
        impl crate::vm::value::type_aliases::$impl_var {
            pub fn get_override_fn(self, name: &'static str) -> Option<crate::vm::value::BuiltinFn> {
                $(
                    fn $fn_name(
                        v: Vec<crate::vm::interpreter::ValueKey>,
                        vm: &mut crate::vm::interpreter::Vm,
                        area: crate::sources::CodeArea
                    ) -> crate::vm::interpreter::RuntimeResult<crate::vm::value::Value> {

                        let mut arg_idx = 0usize;

                        $(
                            paste::paste! {
                                $(
                                    mod $arg_name {
                                        use crate::vm::value::gen_wrapper;

                                        $(
                                            $(
                                                pub struct [<$ref_ty Getter>](pub crate::vm::interpreter::ValueKey);

                                                gen_wrapper! {
                                                    pub struct [<$ref_ty Ref>]: & $ref_ty
                                                }
                                                gen_wrapper! {
                                                    pub struct [<$ref_ty MutRef>]: mut & $ref_ty
                                                }

                                                impl [<$ref_ty Getter>] {
                                                    pub fn get_ref(&self, vm: &crate::vm::interpreter::Vm) -> [<$ref_ty Ref>]<'_> {
                                                        todo!()
                                                        // match &vm.memory[self.0].value {
                                                        //     Value::String(s) => StringRef(s)
                                                        //     _ => panic!("valuekey does not point to value of correct type !!!!!!!!")
                                                        // }
                                                    }
                                                    pub fn get_mut_ref(&self, vm: &mut crate::vm::interpreter::Vm) -> [<$ref_ty MutRef>]<'_> {
                                                        todo!()
                                                        // match &vm.memory[self.0].value {
                                                        //     Value::String(s) => StringMutRef(s)
                                                        //     _ => panic!("valuekey does not point to value of correct type !!!!!!!!")
                                                        // }
                                                    }
                                                }
                                            )?
                                        )+

                                        impl_type! {
                                            @gen_wrapper [<$arg_name:camel>] $( $(&$ref_ty)? $($deref_ty)? )|+
                                        }
                                    }

                                    #[allow(clippy::let_unit_value)]
                                    let $arg_name = match vm.memory[v[arg_idx]].value {
                                        $(
                                            $(
                                                crate::vm::value::Value::$ref_ty{..} => $arg_name::[<$arg_name:camel>]::$ref_ty($arg_name::[<$ref_ty Getter>](v[arg_idx])),
                                            )?
                                        )+
                                        _ => unreachable!(),
                                    };

                                )?
                                $(
                                    let crate::vm::value::Value::$arg_name
                                        $(
                                            ( $( $v_val ),* )
                                        )?
                                        $(
                                            { $( $v_n $(: $v_val_s)? ,)* }
                                        )?
                                    = vm.memory[v[arg_idx]].value.clone() else {
                                        unreachable!();
                                    };
                                )?
                            }
                            arg_idx += 1;
                        )*
                    }
                )*

                match name {
                    $(
                        stringify!($fn_name) => Some(crate::vm::value::BuiltinFn(&$fn_name)),
                    )*
                    _ => None
                }
            }
            pub fn get_override_const(self, name: &'static str) -> Option<crate::compiling::bytecode::Constant> {
                None
            }
        }

        paste::paste! {
            #[cfg(test)]
            mod [<$impl_var:snake _core_gen>] {
                #[test]
                pub fn [<$impl_var:snake _core_gen>]() {
                    let path = std::path::PathBuf::from(format!("{}{}.spwn", crate::CORE_PATH, stringify!( [<$impl_var:snake>] )));
                    let out = indoc::formatdoc!(r#"
                            /* 
                             * This file is automatically generated!
                             * Do not modify or your changes will be overwritten!  
                            */
                            {impl_raw}
                            #[doc(u{impl_doc:?})]
                            impl @{typ} {{{consts}
                                {macros}
                            }}
                        "#,
                        impl_raw = stringify!($($impl_raw),*),
                        impl_doc = <[String]>::join(&[$($impl_doc .to_string()),*], "\n"),
                        typ = stringify!( [<$impl_var:snake>] ),
                        consts = "",
                        macros = "",
                    );

                    std::fs::write(path, &out).unwrap();
                }
            }

        }
    };

    (@gen_wrapper $name:ident $(&$ref_ty:ident)? $($deref_ty:ident)?) => {
        $(
            gen_wrapper! {
                pub struct $name: *$deref_ty
            }
        )?
        $(
            paste::paste! {
                pub type $name = [<$ref_ty Getter>];
            }
        )?
    };
    (@gen_wrapper $name:ident $( $(&$ref_ty:ident)? $($deref_ty:ident)? )|+) => {
        paste::paste! {
            gen_wrapper! {
                pub enum $name: $( $($deref_ty |)? )+; $( $( $ref_ty( [<$ref_ty Getter>] ) ,)? )+
            }
        }
    };
}

impl_type! {
    impl String {
        Constants:

        Functions:
        fn poo(
            String(s) as self = r#"bunkledo"#,
            arg1: Int | &Range = 10,
            arg2: Int,
            Range(start, end, step) as arg2 where Key(b_k),
            arg4: &String | Float,
        ) -> Range {
            // block
        }
    }
}
