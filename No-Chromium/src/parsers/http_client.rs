use std::error::Error;

pub fn fetch_html(url: &str) -> Result<String, Box<dyn Error>> {
    let response = reqwest::blocking::get(url)?;
    let text = response.text()?;
    Ok(text)
}
