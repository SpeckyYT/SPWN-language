use std::string::ToString;

use super::context::CallInfo;
use super::interpreter::Vm;
use super::pattern::ConstPattern;
use super::value::ValueType;
use crate::error_maker;
use crate::parsing::utils::operators::{BinOp, UnaryOp};
use crate::sources::CodeArea;
use crate::util::hyperlink;

error_maker! {
    Title: "Runtime Error"
    Extra: {
        vm: &Vm,
    }
    pub enum RuntimeError {
        /////////
        #[
            Message: "Invalid operands", Note: None;
            Labels: [
                area => "Invalid operands for `{}` operator": op.to_str();
                a.1 => "This is of type {}": a.0.runtime_display(vm);
                b.1 => "This is of type {}": b.0.runtime_display(vm);
            ]
        ]
        InvalidOperands {
            a: (ValueType, CodeArea),
            b: (ValueType, CodeArea),
            op: BinOp,
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Invalid unary operand", Note: None;
            Labels: [
                area => "Invalid operand for `{}` unary operator": op.to_str();
                v.1 => "This is of type {}": v.0.runtime_display(vm);
            ]
        ]
        InvalidUnaryOperand {
            v: (ValueType, CodeArea),
            op: UnaryOp,
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Type mismatch", Note: None;
            Labels: [
                area => "Expected {}, found {}": expected.runtime_display(vm), v.0.runtime_display(vm);
                v.1 => "Value defined as {} here": v.0.runtime_display(vm);
            ]
        ]
        TypeMismatch {
            v: (ValueType, CodeArea),
            area: CodeArea,
            expected: ValueType,
            [call_stack]
        },

        /////////
        #[
            Message: "Cannot convert between types", Note: None;
            Labels: [
                area => "Cannot convert {} to {}": v.0.runtime_display(vm), to.runtime_display(vm);
                v.1 => "This is of type {}": v.0.runtime_display(vm);
            ]
        ]
        CannotConvertType {
            v: (ValueType, CodeArea),
            to: ValueType,
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Cannot iterator", Note: None;
            Labels: [
                area => "Cannot iterate over {}": v.0.runtime_display(vm);
                v.1 => "Value defined as {} here": v.0.runtime_display(vm);
            ]
        ]
        CannotIterate {
            v: (ValueType, CodeArea),
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Cannot instance builtin type", Note: None;
            Labels: [
                area => "Cannot instance builtin type {}": typ.runtime_display(vm);
            ]
        ]
        CannotInstanceBuiltinType {
            area: CodeArea,
            typ: ValueType,
            [call_stack]
        },

        /////////
        #[
            Message: "Invalid object value", Note: None;
            Labels: [
                v.1 => "{} is not a valid object value": v.0;
                area => "Object key used here";
            ]
        ]
        InvalidObjectValue {
            v: (String, CodeArea),
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Too many arguments", Note: None;
            Labels: [
                call_area => "Received {} arguments, expected {}": call_arg_amount, macro_arg_amount;
                macro_def_area => "Macro defined to take {} arguments here": macro_arg_amount;
            ]
        ]
        TooManyArguments {
            call_area: CodeArea,
            macro_def_area: CodeArea,
            macro_arg_amount: usize,
            call_arg_amount: usize,
            [call_stack]
        },

        /////////
        #[
            Message: "Invalid keyword argument", Note: None;
            Labels: [
                call_area => "Keyword argument `{}` is invalid": arg_name;
                macro_def_area => "Macro defined here";
            ]
        ]
        InvalidKeywordArgument {
            call_area: CodeArea,
            macro_def_area: CodeArea,
            arg_name: String,
            [call_stack]
        },

        /////////
        #[
            Message: "Argument not satisfied", Note: None;
            Labels: [
                call_area => "Argument `{}` not satisfied": arg_name;
                macro_def_area => "Macro defined here";
            ]
        ]
        ArgumentNotSatisfied {
            call_area: CodeArea,
            macro_def_area: CodeArea,
            arg_name: String,
            [call_stack]
        },

        /////////
        #[
            Message: "Argument pattern mismatch", Note: None;
            Labels: [
                call_area => "Call occurred here";
                macro_def_area => "Argument `{}` was defined as taking {} here": arg_name, pattern.runtime_display(vm);
                v.1 => "This `{}` is not {}": v.0.runtime_display(vm), pattern.runtime_display(vm);
            ]
        ]
        ArgumentPatternMismatch {
            call_area: CodeArea,
            macro_def_area: CodeArea,
            arg_name: String,
            pattern: ConstPattern,
            v: (ValueType, CodeArea),
            [call_stack]
        },

        /////////
        #[
            Message: "Nonexistent member", Note: None;
            Labels: [
                area => "Member `{}` does not exist on this {}": member, base_type.runtime_display(vm);
            ]
        ]
        NonexistentMember {
            area: CodeArea,
            member: String,
            base_type: ValueType,
            [call_stack]
        },

        /////////
        #[
            Message: "Tried to access private member", Note: None;
            Labels: [
                area => "Member `{}` is private": member;
            ]
        ]
        PrivateMemberAccess {
            area: CodeArea,
            member: String,
            [call_stack]
        },

        /////////
        #[
            Message: "Nonexistent associated member", Note: None;
            Labels: [
                area => "Associated member `{}` does not exist on {}": member, base_type.runtime_display(vm);
            ]
        ]
        NonexistentAssociatedMember {
            area: CodeArea,
            member: String,
            base_type: ValueType,
            [call_stack]
        },

        /////////
        #[
            Message: "Associated function is not method", Note: None;
            Labels: [
                area => "Associated function `{}` is not a method, because it does not have a `self` argument": func_name;
                def_area => "Associated function defined on type {} here": base_type.runtime_display(vm);
            ]
        ]
        AssociatedNotAMethod {
            area: CodeArea,
            def_area: CodeArea,
            func_name: String,
            base_type: ValueType,
            [call_stack]
        },

        /////////
        #[
            Message: "Associated member is not method", Note: None;
            Labels: [
                area => "Member `{}` implemented on type {} is not a method": member_name, base_type.runtime_display(vm);
                def_area => "Member defined as {} here": member_type.runtime_display(vm);
            ]
        ]
        NotAMethod {
            area: CodeArea,
            def_area: CodeArea,
            member_name: String,
            member_type: ValueType,
            base_type: ValueType,
            [call_stack]
        },

        /////////
        #[
            Message: "Nonexistent type member", Note: None;
            Labels: [
                area => "Type {} does not exist in this module": format!("@{type_name}");
            ]
        ]
        NonexistentTypeMember {
            area: CodeArea,
            type_name: String,
            [call_stack]
        },

        /////////
        #[
            Message: "Tried to access private type", Note: None;
            Labels: [
                area => "Type {} is private": format!("@{type_name}");
            ]
        ]
        PrivateType {
            area: CodeArea,
            type_name: String,
            [call_stack]
        },

        /////////
        #[
            Message: "Invalid index", Note: None;
            Labels: [
                area => "{} cannot be indexed by {}": base.0.runtime_display(vm), index.0.runtime_display(vm);
                base.1 => "This is of type {}": base.0.runtime_display(vm);
                index.1 => "This is of type {}": index.0.runtime_display(vm);
            ]
        ]
        InvalidIndex {
            base: (ValueType, CodeArea),
            index: (ValueType, CodeArea),
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Index out of bounds", Note: None;
            Labels: [
                area => "Index {} is out of bounds for this {} of length {}": index, typ.runtime_display(vm), len;
            ]
        ]
        IndexOutOfBounds {
            len: usize,
            index: i64,
            area: CodeArea,
            typ: ValueType,
            [call_stack]
        },

        /////////
        #[
            Message: "Assertion failed", Note: None;
            Labels: [
                area => "Assertion happened here";
            ]
        ]
        AssertionFailed {
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Equality assertion failed", Note: None;
            Labels: [
                area => "{} is not equal to {}": left, right;
            ]
        ]
        EqAssertionFailed {
            area: CodeArea,
            left: String,
            right: String,
            [call_stack]
        },

        /////////
        #[
            Message: "Added object in runtime context", Note: Some("TODO (link to docs)".into());
            Labels: [
                area => "Cannot add this object at runtime";
            ]
        ]
        AddObjectInTriggerContext {
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Runtime Error", Note: None;
            Labels: [
                area => "{}": message;
            ]
        ]
        ThrownError {
            area: CodeArea,
            message: String,
            [call_stack]
        },
    }
}
