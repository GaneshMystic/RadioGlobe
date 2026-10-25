mod stations;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let list = stations::load()?;
    println!("{} stations loaded", list.len());
    Ok(())
}
