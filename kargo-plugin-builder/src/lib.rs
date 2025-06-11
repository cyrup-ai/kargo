use clap::{Arg, Command};
use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
use std::io::Read;
use std::sync::Arc;

use gag::BufferRedirect;
use regex::{Regex, RegexSet};

type Handler = Arc<dyn Fn(ExecutionContext) -> BoxFuture + Send + Sync>;
type OutputHandler = Arc<dyn Fn(&regex::Match<'_>, &ExecutionContext) -> BoxFuture + Send + Sync>;

pub struct PluginBuilder {
    name: String,
    about: Option<String>,
    args: Vec<Arg>,
    run: Option<Handler>,
    patterns: Vec<(String, OutputHandler)>,
}

impl PluginBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            about: None,
            args: vec![],
            run: None,
            patterns: vec![],
        }
    }

    pub fn about(mut self, txt: impl Into<String>) -> Self {
        self.about = Some(txt.into());
        self
    }
    pub fn arg(mut self, a: Arg) -> Self {
        self.args.push(a);
        self
    }

    pub fn on_execute<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(ExecutionContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        self.run = Some(Arc::new(move |ctx| Box::pin(f(ctx))));
        self
    }

    /// Expectrl-style trigger
    pub fn on_match<F, Fut>(mut self, pattern: impl AsRef<str>, handler: F) -> Self
    where
        F: Fn(&regex::Match<'_>, &ExecutionContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        self.patterns.push((
            pattern.as_ref().to_owned(),
            Arc::new(move |m, c| Box::pin(handler(m, c))),
        ));
        self
    }

    pub fn build(self) -> Box<dyn PluginCommand> {
        let Self {
            name,
            about,
            args,
            run,
            patterns,
        } = self;
        let run = run.expect("PluginBuilder::on_execute not called");

        struct Impl {
            clap: Command,
            run: Handler,
            set: RegexSet,
            regs: Vec<Regex>,
            cbs: Vec<OutputHandler>,
        }
        impl PluginCommand for Impl {
            fn clap(&self) -> Command {
                self.clap.clone()
            }
            fn run(&self, ctx: ExecutionContext) -> BoxFuture {
                let run_closure = Arc::clone(&self.run);
                let set = self.set.clone();
                let regs = self.regs.clone();
                let cbs = self.cbs.clone();
                Box::pin(async move {
                    // capture stdout while running
                    let mut stdout_buf = BufferRedirect::stdout()?;
                    let result = run_closure(ctx.clone()).await;
                    let mut out = String::new();
                    stdout_buf.read_to_string(&mut out)?;
                    drop(stdout_buf);

                    // print back what we captured
                    print!("{}", out);
                    result?;

                    // now run pattern matches
                    for idx in set.matches(&out).into_iter() {
                        let re = &regs[idx];
                        let cb = &cbs[idx];
                        for m in re.find_iter(&out) {
                            cb(&m, &ctx).await?;
                        }
                    }
                    Ok(())
                })
            }
        }

        // ── pre-compile patterns once ──────────────────────────────────
        let (pat_strings, cbs): (Vec<_>, Vec<_>) = patterns.into_iter().unzip();
        let regs: Vec<Regex> = pat_strings
            .iter()
            .map(|p| Regex::new(p).expect("invalid regex"))
            .collect();
        let set = if pat_strings.is_empty() {
            RegexSet::empty()
        } else {
            RegexSet::new(&pat_strings).expect("regex set build failed")
        };

        let mut cmd = Command::new(name);
        if let Some(a) = about {
            cmd = cmd.about(a);
        }
        for arg in args {
            cmd = cmd.arg(arg);
        }

        Box::new(Impl {
            clap: cmd,
            run,
            set,
            regs,
            cbs,
        })
    }
}
