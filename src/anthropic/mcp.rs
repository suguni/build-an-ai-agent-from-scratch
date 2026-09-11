use std::borrow::Cow;
use std::ops::Deref;
use crate::anthropic::tools::{Tool, ToolResult, ToolSpec, ToolUse};
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};
use std::pin::Pin;
use tokio::process::Command;

pub fn load_tools() {}

struct StdioMcpClient {
    client: RunningService<RoleClient, ()>,
}

impl StdioMcpClient {
    pub async fn connect(command: Command) -> anyhow::Result<Self> {
        let process = TokioChildProcess::new(command)?;
        let client = ().serve(process).await?;
        Ok(Self { client })
    }

    pub async fn list_tools(&self) -> anyhow::Result<Vec<Box<impl Tool>>> {
        Ok(self
            .client
            .list_all_tools()
            .await?
            .into_iter()
            .map(|tool| Box::new(McpTool { tool }))
            .collect::<Vec<Box<_>>>())
    }
}

struct McpTool {
    tool: rmcp::model::Tool,
}

impl Tool for McpTool {
    fn name(&self) -> Cow<'static, str> {
        self.tool.name.clone()
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.tool.name,
            description: self.tool.description.clone(),
            input_schema: serde_json::json!(self.tool.input_schema),
        }
    }

    fn run<'a>(
        &'a self,
        tool_use: ToolUse,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ToolResult>> + Send + 'a>> {
        todo!()
    }
}
