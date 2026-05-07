use crate::cli::PluginArgs;
use crate::error::UswitchError;
use crate::output;
use crate::plugin;

pub fn execute(args: PluginArgs) -> Result<(), UswitchError> {
    let name = args.name.as_deref();

    match name {
        Some(n) => show_plugin(n),
        None => list_plugins(),
    }
}

fn list_plugins() -> Result<(), UswitchError> {
    let plugins = plugin::load_all()?;

    if plugins.is_empty() {
        output::print_empty(
            "No plugins found",
            &format!("The plugins directory is empty: {}", plugin::PLUGINS_DIR),
            "Hint: Add .toml plugin manifests to the plugins directory",
        );
        return Ok(());
    }

    output::print_header("Plugins");
    output::print_info(&format!("Directory: {}", plugin::PLUGINS_DIR));
    println!();

    let max_name_w = plugins.iter().map(|p| p.name.len()).max().unwrap_or(12).max(12);

    for p in &plugins {
        let installed = if plugin::is_deployed(&p) {
            output::green("●")
        } else {
            output::dim("○")
        };
        let has_bot = if p.telegram.as_ref().map_or(false, |t| !t.package.is_empty()) {
            " 📱"
        } else {
            ""
        };
        println!(
            "  {} {:<name_w$}  {}{}",
            installed,
            p.name,
            p.description,
            has_bot,
            name_w = max_name_w,
        );
        if !has_bot.is_empty() {
            println!(
                "         {:name_w$}  {} {}",
                "",
                output::dim("bot:"),
                p.telegram.as_ref().unwrap().package,
                name_w = max_name_w,
            );
        }
    }

    println!();
    output::print_bullet("Install:  usw install <name>");
    output::print_bullet("Details:  usw plugin <name>");
    println!();

    Ok(())
}

fn show_plugin(name: &str) -> Result<(), UswitchError> {
    let p = plugin::find(name)?;

    output::print_header(&p.name);
    if !p.description.is_empty() {
        println!("  {}", p.description);
    }
    if !p.homepage.is_empty() {
        println!("  {}", output::cyan(&p.homepage));
    }
    println!();

    output::print_section("Install");
    output::print_kv("Method", if p.install.method.is_empty() { "binary" } else { &p.install.method });
    if !p.install.binary_search.is_empty() {
        output::print_section("Search paths");
        for s in &p.install.binary_search {
            let ok = std::path::Path::new(s).exists();
            let icon = if ok { output::green("●") } else { output::red("○") };
            println!("    {} {}", icon, s);
        }
    }
    if !p.install.npm_package.is_empty() {
        output::print_kv("npm", &p.install.npm_package);
    }
    if !p.install.pip_package.is_empty() {
        output::print_kv("pip", &p.install.pip_package);
    }

    println!();
    output::print_section("Runtime");
    if !p.runtime.command.is_empty() {
        output::print_kv("Command", &format!("{} {}", p.runtime.command, p.runtime.args.join(" ")));
    }
    if p.runtime.port > 0 {
        output::print_kv("Port", &p.runtime.port.to_string());
    }
    if !p.runtime.work_dir.is_empty() {
        output::print_kv("WorkDir", &p.runtime.work_dir);
    }
    if !p.runtime.env.is_empty() {
        output::print_section("Environment");
        for (k, v) in &p.runtime.env {
            println!("    {}={}", k, v);
        }
    }

    if let Some(ref bot) = p.telegram {
        if !bot.package.is_empty() {
            println!();
            output::print_section("Telegram Bot");
            output::print_kv("Package", &bot.package);
            output::print_kv("API URL", &bot.api_url);
            output::print_kv("Provider", &bot.provider);
            output::print_kv("Model", &bot.model_id);
        }
    }

    println!();
    let installed = plugin::is_deployed(&p);
    if installed {
        output::print_success(&format!("Status: installed"));
    } else {
        output::print_info(&format!("Status: not installed. Run: usw install {}", name));
    }
    println!();

    Ok(())
}
