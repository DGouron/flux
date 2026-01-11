use anyhow::Result;
use flux_core::{Config, SuggestionReason, SuggestionReport, Translator};

pub fn list() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let translator = Translator::new(config.general.language);

    let report = SuggestionReport::load().unwrap_or_default();

    if report.suggestions.is_empty() {
        println!("\n{}\n", translator.get("command.suggestions_empty"));
        return Ok(());
    }

    println!("\n{}:\n", translator.get("command.suggestions_header"));

    for (index, suggestion) in report.suggestions.iter().enumerate() {
        let prefix = if index == report.suggestions.len() - 1 {
            "└──"
        } else {
            "├──"
        };

        let reason_text = match suggestion.reason {
            SuggestionReason::FrequentShortBursts => translator.format(
                "command.suggestions_reason_short_bursts",
                &[("count", &suggestion.short_burst_count.to_string())],
            ),
        };

        println!(
            "{} {} ({})",
            prefix, suggestion.application_name, reason_text
        );
    }

    if report.context_switch_count > 0 {
        println!(
            "\n{}",
            translator.format(
                "command.suggestions_context_switches",
                &[("count", &report.context_switch_count.to_string())]
            )
        );
    }

    println!("\n{}", translator.get("command.suggestions_hint"));

    println!();
    Ok(())
}

pub fn clear() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let translator = Translator::new(config.general.language);

    SuggestionReport::clear()?;

    println!("{}", translator.get("command.suggestions_cleared"));
    Ok(())
}
