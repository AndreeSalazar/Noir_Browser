pub struct SocksProxy {}
impl SocksProxy {
    pub fn new() -> Self { Self {} }
    pub async fn run(&self) -> anyhow::Result<()> { Ok(()) }
}
