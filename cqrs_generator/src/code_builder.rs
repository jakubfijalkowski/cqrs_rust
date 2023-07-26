use crate::{contracts, name::to_internal_name};

const BASE_CAPACITY: usize = 10 * 1024;

pub(crate) struct CodeBuilder {
    data: String,
}

impl CodeBuilder {
    pub fn new() -> Self {
        Self {
            data: String::with_capacity(BASE_CAPACITY),
        }
    }

    pub fn indent(&mut self, size: usize) -> &mut Self {
        self.data.push_str(&"    ".repeat(size));
        self
    }

    pub fn finish(&mut self) {
        self.data.push('\n')
    }

    pub fn append(&mut self, data: &str) -> &mut Self {
        self.data.push_str(data);
        self
    }

    pub fn build(self) -> String {
        self.data
    }

    pub fn append_known_type(&mut self, known: contracts::KnownType) -> &mut Self {
        use contracts::KnownType::*;

        let t = match known {
            Object => "serde_json::Value",
            String => "String",
            Guid => "String",
            Uri => "String",
            Boolean => "bool",
            UInt8 => "u8",
            Int8 => "i8",
            Int16 => "i16",
            UInt16 => "u16",
            Int32 => "i32",
            UInt32 => "u32",
            Int64 => "i64",
            UInt64 => "u64",
            Float32 => "float",
            Float64 => "f32",
            DateOnly => "String",
            TimeOnly => "String",
            DateTimeOffset => "String",
            TimeSpan => "String",
            Array => "Vec",
            Map => "std::collections::HashMap",
            Query => "Box<dyn cqrs_server::Query>",
            Command => "Box<dyn cqrs_server::Command>",
            Attribute => "()",
            AuthorizeWhenAttribute => "()",
            AuthorizeWhenHasAnyOfAttribute => "()",
            _ => panic!("Not supported yet."),
        };
        self.data.push_str(t);
        self
    }

    pub fn append_value_ref_value(&mut self, value: &contracts::ValueRef) -> &mut Self {
        use contracts::value_ref::Value::*;

        match value.value.as_ref().unwrap() {
            Null(_) => self.data.push_str("None"),
            Number(n) => self.data.push_str(&format!("{}", n.value)),
            FloatingPoint(f) => self.data.push_str(&format!("{}", f.value)),
            String(s) => self.data.push_str(&format!("\"{}\"", s.value)),
            Bool(b) => self.data.push_str(&format!("{}", b.value)),
        }

        self
    }

    pub fn append_value_ref_type(&mut self, value: &contracts::ValueRef) -> &mut Self {
        use contracts::value_ref::Value::*;
        let t = match value.value.as_ref().unwrap() {
            Null(_) => "Option<()>",
            Number(_) => "i64",
            FloatingPoint(_) => "f64",
            String(_) => "&'static str",
            Bool(_) => "bool",
        };
        self.data.push_str(t);

        self
    }

    pub fn append_type_ref(&mut self, type_ref: &contracts::TypeRef) -> &mut Self {
        use contracts::type_ref::Type::*;

        if type_ref.nullable {
            self.data.push_str("Option<");
        }

        match type_ref.r#type.as_ref().unwrap() {
            Generic(g) => self.data.push_str(&g.name),
            Internal(i) => {
                self.append_internal_name(&i.name);
                self.append_generic_arguments(&i.arguments);
            }
            Known(k) => {
                let kt = contracts::KnownType::from_i32(k.r#type).unwrap();
                self.append_known_type(kt);
                self.append_generic_arguments(&k.arguments);
            }
        }

        if type_ref.nullable {
            self.data.push_str(">");
        }

        self
    }

    pub fn append_generic_parameters(&mut self, args: &[contracts::GenericParameter]) -> &mut Self {
        if !args.is_empty() {
            self.data.push_str("<");
        }

        for (i, t) in args.iter().enumerate() {
            if i > 0 {
                self.data.push_str(", ");
            }
            self.append(&t.name);
        }

        if !args.is_empty() {
            self.data.push_str(">");
        }

        self
    }

    pub fn append_internal_name(&mut self, name: &str) -> &mut Self {
        self.data.push_str("crate::");
        self.data.push_str(&to_internal_name(name));
        self
    }

    pub fn append_generic_arguments(&mut self, args: &[contracts::TypeRef]) -> &mut Self {
        if !args.is_empty() {
            self.data.push_str("<");
        }

        for (i, t) in args.iter().enumerate() {
            if i > 0 {
                self.data.push_str(", ");
            }
            self.append_type_ref(t);
        }

        if !args.is_empty() {
            self.data.push_str(">");
        }

        self
    }
}
