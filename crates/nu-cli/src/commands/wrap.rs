use crate::commands::WholeStreamCommand;
use crate::context::CommandRegistry;
use crate::prelude::*;
use indexmap::IndexMap;
use nu_errors::ShellError;
use nu_protocol::{ReturnSuccess, Signature, SyntaxShape, UntaggedValue, Value};
use nu_source::Tagged;

pub struct Wrap;

#[derive(Deserialize)]
struct WrapArgs {
    column: Option<Tagged<String>>,
}

impl WholeStreamCommand for Wrap {
    fn name(&self) -> &str {
        "wrap"
    }

    fn signature(&self) -> Signature {
        Signature::build("wrap").optional(
            "column",
            SyntaxShape::String,
            "the name of the new column",
        )
    }

    fn usage(&self) -> &str {
        "Wraps the given data in a table."
    }

    fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        // args.process(registry, wrap)?.run()
        wrap(args, registry)
    }

    fn examples(&self) -> &[Example] {
        &[
            Example {
                description: "Wrap a list into a table with the default column name",
                example: "echo [1 2 3] | wrap",
            },
            Example {
                description: "Wrap a list into a table with a given column name",
                example: "echo [1 2 3] | wrap MyColumn",
            },
        ]
    }
}

fn wrap(args: CommandArgs, registry: &CommandRegistry) -> Result<OutputStream, ShellError> {
    let registry = registry.clone();

    let stream = async_stream! {
        let (WrapArgs { column }, mut input) = args.process(&registry).await?;
        let mut result_table = vec![];
        let mut are_all_rows = true;

        while let Some(value) = input.next().await {
            match value {
                Value {
                    value: UntaggedValue::Row(_),
                    ..
                } => {
                    result_table.push(value);
                }
                _ => {
                    are_all_rows = false;

                    let mut index_map = IndexMap::new();
                    index_map.insert(
                        match &column {
                            Some(key) => key.item.clone(),
                            None => "Column".into(),
                        },
                        value,
                    );

                    result_table.push(UntaggedValue::row(index_map).into_value(Tag::unknown()));
                }

            }
        }

        if are_all_rows {
            let mut index_map = IndexMap::new();
            index_map.insert(
                match &column {
                    Some(key) => key.item.clone(),
                    None => "Column".into(),
                },
                UntaggedValue::table(&result_table).into_value(Tag::unknown()),
            );

            let row = UntaggedValue::row(index_map).into_untagged_value();

            yield ReturnSuccess::value(row);
        } else {
            for item in result_table
                .iter()
                .map(|row| ReturnSuccess::value(row.clone())) {

                yield item;
            }
        }
    };

    Ok(stream.to_output_stream())
}
