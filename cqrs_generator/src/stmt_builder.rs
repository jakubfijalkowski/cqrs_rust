use crate::{
    code_builder::CodeBuilder,
    contracts,
    name::{get_type, to_const, to_field, to_namespace, to_type},
};

pub struct StmtBuilder {
    builder: CodeBuilder,
    indent: usize,
}

impl StmtBuilder {
    pub fn new() -> Self {
        let builder = CodeBuilder::new();
        Self { builder, indent: 0 }
    }

    pub fn descend(&mut self, namespace: &str) {
        self.line().append("#[allow(unused_imports, dead_code)]").finish();
        self.line()
            .append("pub mod ")
            .append(namespace)
            .append(" {")
            .finish();
        self.indent();
        self.line()
            .append("use serde::{Serialize, Deserialize};")
            .finish();
        self.line()
            .append("use serde_repr::{Serialize_repr, Deserialize_repr};")
            .finish();
    }

    pub fn go_up(&mut self) {
        self.dedent();
        self.line().append("}").finish();
    }

    pub fn build(self) -> String {
        self.builder.build()
    }

    fn line(&mut self) -> &mut CodeBuilder {
        self.builder.indent(self.indent)
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
    }
}

impl StmtBuilder {
    pub fn append_statemet(&mut self, stmt: &contracts::Statement) {
        use contracts::statement::Content::*;

        match stmt.content.as_ref().unwrap() {
            Enum(r#enum) => self.append_enum(stmt, r#enum),
            Dto(dto) => self.append_dto(stmt, dto),
            Query(query) => self.append_query(stmt, query),
            Command(command) => self.append_command(stmt, command),
            _ => {}
        }
    }

    pub fn append_enum(
        &mut self,
        stmt: &contracts::Statement,
        r#enum: &contracts::statement::Enum,
    ) {
        self.line()
            .append("#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr, PartialEq)]")
            .finish();
        self.line().append("#[repr(i64)]").finish();
        self.line().append("#[allow(non_camel_case_types)]").finish();

        self.line()
            .append("pub enum ")
            .append(&to_type(get_type(&stmt.name)))
            .append(" {")
            .finish();
        self.indent();

        for m in r#enum.members.iter() {
            self.line()
                .append(&m.name)
                .append(" = ")
                .append(&m.value.to_string())
                .append(",")
                .finish();
        }

        self.dedent();
        self.line().append("}").finish();
    }

    pub fn append_dto(&mut self, stmt: &contracts::Statement, dto: &contracts::statement::Dto) {
        self.append_type(dto.type_descriptor.as_ref().unwrap(), stmt)
    }

    pub fn append_query(
        &mut self,
        stmt: &contracts::Statement,
        query: &contracts::statement::Query,
    ) {
        let descr = query.type_descriptor.as_ref().unwrap();
        let name = get_type(&stmt.name);

        self.append_type(descr, stmt);

        self.line()
            .append("impl")
            .append_generic_parameters(&descr.generic_parameters)
            .append(" cqrs_server::Query for ")
            .append(&to_type(name))
            .append(" {")
            .finish();

        self.indent();

        self.line()
            .append("type Result = ")
            .append_type_ref(query.return_type.as_ref().unwrap())
            .append(";")
            .finish();
        self.line().finish();
        self.line().append("fn name() -> &'static str {").finish();

        self.indent();
        self.line()
            .append("\"")
            .append(&stmt.name)
            .append("\"")
            .finish();
        self.dedent();
        self.line().append("}").finish();

        self.dedent();
        self.line().append("}").finish();
    }

    pub fn append_command(
        &mut self,
        stmt: &contracts::Statement,
        command: &contracts::statement::Command,
    ) {
        let descr = command.type_descriptor.as_ref().unwrap();
        let name = get_type(&stmt.name);

        self.append_type(descr, stmt);

        self.line()
            .append("#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr, PartialEq)]")
            .finish();
        self.line().append("#[repr(i64)]").finish();

        self.line()
            .append("pub enum ")
            .append(&to_type(name))
            .append("ErrorCodes")
            .append(" {")
            .finish();
        self.indent();

        for e in command.error_codes.iter() {
            self.append_codes("", e.code.as_ref().unwrap());
        }

        if command.error_codes.is_empty() {
            self.line().append("__MarkerForEmptyErrorCodes = 0,").finish();
        }

        self.dedent();
        self.line().append("}").finish();

        self.line()
            .append("impl")
            .append_generic_parameters(&descr.generic_parameters)
            .append(" cqrs_server::Command for ")
            .append(&to_type(name))
            .append(" {")
            .finish();

        self.indent();

        self.line()
            .append("type ErrorCodes = ")
            .append(&to_type(name))
            .append("ErrorCodes")
            .append(";")
            .finish();
        self.line().finish();
        self.line().append("fn name() -> &'static str {").finish();

        self.indent();
        self.line()
            .append("\"")
            .append(&stmt.name)
            .append("\"")
            .finish();
        self.dedent();
        self.line().append("}").finish();

        self.dedent();
        self.line().append("}").finish();
    }

    fn append_type(&mut self, descr: &contracts::TypeDescriptor, stmt: &contracts::Statement) {
        let type_name = get_type(&stmt.name);
        if !descr.constants.is_empty() {
            let mut ns = to_namespace(type_name);
            ns.push_str("_props");
            self.descend(&ns);

            for c in descr.constants.iter() {
                let value = c.value.as_ref().unwrap();
                self.line()
                    .append("pub const ")
                    .append(&to_const(&c.name))
                    .append(": ")
                    .append_value_ref_type(&value)
                    .append(" = ")
                    .append_value_ref_value(&value)
                    .append(";")
                    .finish();
            }

            self.go_up();
        }

        self.line()
            .append("#[derive(Clone, Debug, Serialize, Deserialize)]")
            .finish();
        self.line()
            .append("#[serde(rename_all = \"PascalCase\")]")
            .finish();
        self.line()
            .append("pub struct ")
            .append(&to_type(type_name))
            .append_generic_parameters(&descr.generic_parameters)
            .append(" {")
            .finish();
        self.indent();

        for p in descr.extends.iter() {
            let inner_type = p.r#type.as_ref().unwrap();
            let contracts::type_ref::Type::Internal(internal) = inner_type else {
                continue;
            };

            let type_name = get_type(&internal.name);

            self.line()
                .append("pub ")
                .append(&to_field(type_name))
                .append(": ")
                .append_internal_name(&internal.name)
                .append_generic_arguments(&internal.arguments)
                .append(", ")
                .finish();
        }

        for p in descr.properties.iter() {
            self.line()
                .append("pub ")
                .append(&to_field(&p.name))
                .append(": ")
                .append_type_ref(p.r#type.as_ref().unwrap())
                .append(", ")
                .finish();
        }

        for g in descr.generic_parameters.iter() {
            self.line()
                .append(&to_field(&g.name))
                .append(": ")
                .append("std::marker::PhantomData<")
                .append(&g.name)
                .append(">,")
                .finish();
        }

        self.dedent();

        self.line().append("}").finish();
    }

    fn append_codes(&mut self, prefix: &str, code: &contracts::error_code::Code) {
        match code {
            contracts::error_code::Code::Single(s) => {
                self.line()
                    .append(prefix)
                    .append(&s.name)
                    .append(" = ")
                    .append(&s.code.to_string())
                    .append(",")
                    .finish();
            }
            contracts::error_code::Code::Group(g) => {
                let prefix = format!("{}{}", prefix, g.name);
                for c in g.inner_codes.iter() {
                    self.append_codes(&prefix, c.code.as_ref().unwrap());
                }
            }
        }
    }
}
