#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use openai::{completions::Completion, set_key};
use rand::Rng;
use serde::{Deserialize, Serialize};

const PROMPT: &str = "Give me a twitch username for someone who enjoys gaming. It should not include the word gaming, it should not end with numbers. it should be creative, like a mix of two words or someones name, but absolutely should not just combine two words. Present just the username and no other text.";
const EMPTY: String = String::new();

pub async fn generate_names<const N: usize>() -> anyhow::Result<[String; N]> {
    let mut names = [EMPTY; N];
    let mut i = 0;

    set_key(std::env::var("OPENAI_API_KEY")?);

    while i < N {
        i += 1;

        let mut req = Completion::builder("text-davinci-003")
            .prompt(PROMPT)
            .create()
            .await??;

        names[i - 1] = req.choices.remove(0).text;
        names[i - 1] = names[i - 1].trim_start_matches('\n').to_string();
    }

    Ok(names)
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    #[must_use]
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            r: rng.gen(),
            g: rng.gen(),
            b: rng.gen(),
        }
    }

    #[must_use]
    pub fn generate_light() -> Self {
        let mut color = Self::generate();

        // This could theoretically run forever, but that's good enough for now
        while !color.is_light() {
            color = Self::generate();
        }

        color
    }

    #[must_use]
    pub fn is_light(&self) -> bool {
        let (r, g, b) = {
            let r = f32::from(self.r);
            let g = f32::from(self.g);
            let b = f32::from(self.b);

            (r, g, b)
        };
        let luma = r * 0.2126 + g * 0.7152 + b * 0.0722;

        luma > 128.0
    }
}

impl std::fmt::UpperHex for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X}", self.r)?;
        write!(f, "{:02X}", self.g)?;
        write!(f, "{:02X}", self.b)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_names() -> anyhow::Result<()> {
        let names = generate_names::<4>().await?;
        for name in names {
            assert!(!name.is_empty());
        }

        Ok(())
    }

    #[test]
    fn test_generate_color() {
        let light_color = Color::generate_light();

        assert!(light_color.is_light());
    }
}
