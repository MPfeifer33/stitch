use crate::gather::ProjectContext;
use crate::StitchError;

pub fn print_context(ctx: &ProjectContext, is_json: bool) -> Result<(), StitchError> {
    if is_json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "context": ctx,
        }))?);
    } else {
        println!("stitch rebuild: {}", ctx.project_name);
        println!();

        println!("  Type: {}", ctx.project_type);

        if let Some(ref git) = ctx.git_state {
            println!("  Git: {} @ {}{}", git.branch, git.head_sha,
                if git.dirty_count > 0 { format!(" ({} dirty)", git.dirty_count) } else { String::new() });
        }

        let file_count = ctx.structure.iter().filter(|f| !f.is_dir).count();
        println!("  Files: {} indexed", file_count);

        if !ctx.key_files.is_empty() {
            println!();
            println!("  Key files:");
            for kf in &ctx.key_files {
                println!("    {} — {}", kf.path, kf.reason);
            }
        }

        if !ctx.recent_commits.is_empty() {
            println!();
            println!("  Recent commits:");
            for c in ctx.recent_commits.iter().take(5) {
                println!("    {} {}", c.sha, c.subject);
            }
        }

        println!();
        print_sources_inline(&ctx.evidence_sources);
    }
    Ok(())
}

pub fn print_sources(ctx: &ProjectContext, is_json: bool) -> Result<(), StitchError> {
    if is_json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "sources": ctx.evidence_sources,
        }))?);
    } else {
        println!("stitch sources: {}", ctx.project_name);
        println!();
        print_sources_inline(&ctx.evidence_sources);
    }
    Ok(())
}

pub fn print_brief(brief: &str, is_json: bool) -> Result<(), StitchError> {
    if is_json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "brief": brief,
        }))?);
    } else {
        println!("{brief}");
    }
    Ok(())
}

fn print_sources_inline(sources: &crate::gather::EvidenceSources) {
    println!("  Sources:");
    println!("    {} PROJECT.md", if sources.project_md { "✓" } else { "·" });
    println!("    {} README.md", if sources.readme { "✓" } else { "·" });
    println!("    {} witness", if sources.witness { "✓" } else { "·" });
    println!("    {} latch", if sources.latch { "✓" } else { "·" });
    println!("    {} probe", if sources.probe { "✓" } else { "·" });
    println!("    {} atlas", if sources.atlas { "✓" } else { "·" });
}
