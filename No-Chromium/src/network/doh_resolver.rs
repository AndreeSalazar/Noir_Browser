pub struct StubModule {}
impl StubModule {
    pub fn new() -> Self { Self {} }
    pub async fn run(&self) -> anyhow::Result<()> { Ok(()) }
}
