use std::{io, path::Path};

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_codegen::CodegenReturn;
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_span::UnknownExtension;
use oxc_transformer::{TransformOptions, Transformer, TypeScriptOptions};

pub enum Error {
    Io(io::Error),
    Parse(Vec<OxcDiagnostic>),
    Semantic(Vec<OxcDiagnostic>),
    Transformer(Vec<OxcDiagnostic>),
}

#[derive(Default)]
pub struct Transpiler {
    alloc: Allocator,
}

impl Transpiler {
    pub fn transpile<P: AsRef<Path>>(&mut self, path: P) -> Result<CodegenReturn, Error> {
        self.alloc.reset();

        let path = path.as_ref();
        let source_type = SourceType::from_path(path)?;
        let source_text = std::fs::read_to_string(path)?;

        let ret = Parser::new(&self.alloc, &source_text, source_type).parse();
        if !ret.errors.is_empty() {
            return Err(Error::Parse(ret.errors));
        }

        let mut program = ret.program;

        let ret = SemanticBuilder::new()
            // Estimate transformer will triple scopes, symbols, references
            .with_excess_capacity(2.0)
            .build(&program);
        if !ret.errors.is_empty() {
            return Err(Error::Semantic(ret.errors));
        }

        let scoping = ret.semantic.into_scoping();
        let transform_options = TransformOptions {
            typescript: TypeScriptOptions {
                only_remove_type_imports: true,
                allow_namespaces: true,
                remove_class_fields_without_initializer: true,
                rewrite_import_extensions: Some(oxc_transformer::RewriteExtensionsMode::Rewrite),
                ..Default::default()
            },
            ..Default::default()
        };

        let ret = Transformer::new(&self.alloc, path, &transform_options)
            .build_with_scoping(scoping, &mut program);
        if !ret.errors.is_empty() {
            return Err(Error::Transformer(ret.errors));
        }

        Ok(Codegen::new().build(&program))
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<UnknownExtension> for Error {
    fn from(value: UnknownExtension) -> Self {
        Self::Io(io::Error::other(value))
    }
}
