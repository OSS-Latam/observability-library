use datafusion::logical_expr::Literal;
use datafusion::prelude::{col, current_time, lit, Expr};

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Select(Vec<Expr>),
    GroupBy(Vec<Expr>),
    Aggregate(AggregateType, Vec<ExprValue>),
    Filter(String),
    Literal(String, Expr),
    NewCol(String, Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateType {
    Sum,
    Avg,
    Min,
    Max,
    Count,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprValue(pub String, pub Expr);


/// `TransformationBuilder` is a builder for creating custom data transformations.
///
/// This builder provides methods to add various transformation instructions such as select, group by, aggregate, and filter.
///
/// # Examples
///
/// ```ignore
/// let transformation = TransformationBuilder::new()
///     .select(vec!["id", "value", "category"])
///     .aggregate(AggregateType::Sum, vec!["value"])
///     .group_by(vec!["category"])
///     .build();
/// ```
#[derive(Debug)]
pub struct TransformationBuilder {
    instructions: Vec<Instruction>,
}

impl TransformationBuilder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    pub fn select(mut self, columns: Vec<&str>) -> Self {
        self.instructions.push(Instruction::Select(
            columns.iter().map(|&c| col(c)).collect(),
        ));
        self
    }

    pub fn group_by(mut self, columns: Vec<&str>) -> Self {
        self.instructions.push(Instruction::GroupBy(
            columns.iter().map(|&c| col(c)).collect(),
        ));
        self
    }

    pub fn aggregate(mut self, agg_type: AggregateType, columns: Vec<&str>) -> Self {
        self.instructions.push(Instruction::Aggregate(
            agg_type,
            columns
                .iter()
                .map(|&c| ExprValue(c.to_string(), col(c)))
                .collect(),
        ));
        self
    }

    pub fn filter(mut self, condition: &str) -> Self {
        self.instructions
            .push(Instruction::Filter(condition.to_string()));
        self
    }

    pub fn literal<T: Literal>(mut self, alias: &str, value: T) {
        self.instructions
            .push(Instruction::Literal(alias.to_string(), value.lit()));
    }

    pub fn build(self) -> Transformation {
        Transformation {
            instructions: self.instructions,
        }
    }
}

/// `BuiltInMetricsBuilder` is a builder for creating built-in metric transformations.
///
/// This builder provides methods to create common metric transformations such as counting null values.
///
/// # Examples
///
/// ```ignore
/// let transformation = BuiltInMetricsBuilder::new()
///     .count_null("value", None);
/// ```
#[derive(Debug)]
pub struct BuiltInMetricsBuilder {
    instructions: Vec<Instruction>,
}

impl BuiltInMetricsBuilder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    /// Adds a count null transformation for the specified column.
    ///
    /// # Arguments
    ///
    /// * `column` - The name of the column to count null values in.
    /// * `tags` - Optional tags to include in the transformation.
    ///
    /// # Returns
    ///
    /// A `Transformation` object representing the count null transformation.
    pub fn count_null(&mut self, column: &str, tags: Option<Vec<&str>>) -> Transformation {
        self.instructions
            .push(Instruction::Select(vec![col(column)]));
        self.instructions
            .push(Instruction::Filter(format!("{} is null", column)));
        self.instructions.push(Instruction::Aggregate(
            AggregateType::Count,
            vec![ExprValue("value".to_string(), col(column))],
        ));
        self.instructions.push(Instruction::GroupBy(Vec::new()));
        self.completion_schema(column, tags);
        Transformation {
            instructions: self.instructions.clone(),
        }
    }

    /// Completes the schema for the transformation by adding additional columns.
    ///
    /// # Arguments
    ///
    /// * `column_name` - The name of the column being transformed.
    /// * `tags` - Optional tags to include in the transformation.
    fn completion_schema(&mut self, column_name: &str, tags: Option<Vec<&str>>) {
        self.instructions.push(Instruction::Literal(
            "metric_name".to_string(),
            lit(format!("{}_count_null", column_name)),
        ));
        self.instructions.push(Instruction::Literal(
            "tags".to_string(),
            lit(tags.unwrap_or(Vec::new()).join(",")),
        ));
        self.instructions
            .push(Instruction::NewCol("system_ts".to_string(), current_time()));
        self.instructions
            .push(Instruction::NewCol("event_ts".to_string(), current_time()));
    }
}

// Define the Transformation struct to hold the list of Instructions
#[derive(Debug, PartialEq, Default, Clone)]
pub struct Transformation {
    pub instructions: Vec<Instruction>,
}

#[cfg(test)]
mod tests {
    use super::{AggregateType, Instruction, TransformationBuilder};
    use datafusion::logical_expr::col;

    #[test]
    fn test_build_transformation() {
        let builder = TransformationBuilder::new();
        let transform = builder
            .select(vec!["id", "value", "category"])
            .aggregate(AggregateType::Count, vec!["value"])
            .group_by(vec!["category"])
            .build();
        let expected_instruction =
            Instruction::Select(vec![col("id"), col("value"), col("category")]);

        assert_eq!(transform.instructions.contains(&expected_instruction), true)
    }
}
