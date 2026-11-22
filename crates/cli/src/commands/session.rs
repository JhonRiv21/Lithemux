use anyhow::bail;
use lithemux_core::modules::category::{FsDirectories, locate_directory};
use lithemux_core::modules::context::{ContextPorts, build_session_context};
use lithemux_core::modules::launch::ensure_closed;
use lithemux_core::modules::session::{
    Kind, NewSession, Profile, SessionRef, create_session, get_session, remove_session,
    set_keep_alive, set_kind, set_model, set_role,
};
use lithemux_core::platform::clock::system_clock::now_unix;

use crate::args::{NewArgs, ProfileArg, SetArgs};
use crate::output;
use crate::prompt;
use crate::wiring::Wiring;

pub fn new(args: NewArgs) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    let target = SessionRef::parse(&args.target)?;
    let (category, session) = create_session(
        &wiring.categories(),
        &wiring.sessions(),
        NewSession {
            target: target.clone(),
            role: args.role,
            kind: if args.mother {
                Kind::Mother
            } else {
                Kind::Child
            },
            profile: match args.profile {
                ProfileArg::Claude => Profile::Claude,
                ProfileArg::Shell => Profile::Shell,
            },
            cwd: args
                .cwd
                .as_deref()
                .map(|cwd| locate_directory(&FsDirectories, cwd))
                .transpose()?,
            model: args.model,
        },
        now_unix(),
    )?;
    output::session::created(&target, &category, &session);
    Ok(())
}

pub fn set(args: SetArgs) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    let (categories, sessions) = (wiring.categories(), wiring.sessions());
    let target = SessionRef::parse(&args.target)?;
    let (category, session) = get_session(&categories, &sessions, &target)?;
    let mut changed = false;
    if let Some(role) = args.role.as_deref() {
        set_role(&sessions, &session, role)?;
        changed = true;
    }
    if let Some(mother) = args.mother {
        let kind = if mother { Kind::Mother } else { Kind::Child };
        set_kind(&sessions, &category, &session, kind)?;
        changed = true;
    }
    if let Some(keep_alive) = args.keep_alive {
        set_keep_alive(&sessions, &session, keep_alive)?;
        changed = true;
    }
    if args.model.is_some() || args.default_model {
        set_model(&sessions, &session, args.model.as_deref())?;
        changed = true;
    }
    if !changed {
        bail!("nothing to change: pass --role, --mother, --keep-alive, --model or --default-model");
    }
    let (category, session) = get_session(&categories, &sessions, &target)?;
    output::session::updated(&target, &category, &session);
    Ok(())
}

pub fn remove(target: &str, yes: bool) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    let target = SessionRef::parse(target)?;
    let (_, session) = get_session(&wiring.categories(), &wiring.sessions(), &target)?;
    // Removed under a running `claude`, it would go on, out of sight.
    ensure_closed(&wiring.locks(), &session.id)?;
    if !yes && !prompt::confirm(&format!("Remove session \"{target}\"?"))? {
        bail!("not removed (pass --yes to confirm)");
    }
    remove_session(&wiring.sessions(), &session)?;
    output::session::removed(&target, &session);
    Ok(())
}

pub fn context(target: &str) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    let (categories, sessions) = (wiring.categories(), wiring.sessions());
    let (_, session) = get_session(&categories, &sessions, &SessionRef::parse(target)?)?;
    let ports = ContextPorts {
        categories: &categories,
        sessions: &sessions,
        settings: &wiring.settings(),
        boards: &wiring.boards(),
    };
    println!(
        "{}",
        build_session_context(&ports, &session.id, now_unix())?
    );
    Ok(())
}
