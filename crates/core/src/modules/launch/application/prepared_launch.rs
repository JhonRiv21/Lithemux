use crate::modules::category::Category;
use crate::modules::launch::domain::launch_plan::LaunchPlan;
use crate::modules::launch::ports::session_locks::HeldSessionLock;
use crate::modules::session::Session;

/// A session ready to start: the command to run and, for a conversation, the
/// lock that keeps anyone else from starting it. The lock is released when
/// this value is dropped, so whoever starts the process keeps it for as long
/// as the process lives.
pub struct PreparedLaunch {
    category: Category,
    session: Session,
    plan: LaunchPlan,
    /// `None` for a shell session, which has no conversation to protect.
    lock: Option<Box<dyn HeldSessionLock>>,
}

impl PreparedLaunch {
    pub(super) fn new(
        category: Category,
        session: Session,
        plan: LaunchPlan,
        lock: Option<Box<dyn HeldSessionLock>>,
    ) -> Self {
        PreparedLaunch {
            category,
            session,
            plan,
            lock,
        }
    }

    pub fn category(&self) -> &Category {
        &self.category
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// The command to run. It was decided under the lock and is not to be
    /// changed afterwards.
    pub fn plan(&self) -> &LaunchPlan {
        &self.plan
    }

    pub fn holds_a_lock(&self) -> bool {
        self.lock.is_some()
    }

    /// Records the pid of the process that runs the session, so other
    /// terminals can tell who has it open.
    pub fn started(&mut self, pid: u32) {
        if let Some(lock) = self.lock.as_mut() {
            lock.record_pid(pid);
        }
    }
}
