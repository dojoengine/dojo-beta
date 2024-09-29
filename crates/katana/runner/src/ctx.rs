use dojo_test_utils::sequencer::TestSequencer;

use crate::KatanaRunner;

pub enum RunnerCtx {
    Binary(KatanaRunner),
    Embedded(TestSequencer),
}

impl RunnerCtx {
    /// Creates a new [RunnerCtx] from a binary runner handle.
    pub fn binary(handle: KatanaRunner) -> Self {
        Self::Binary(handle)
    }

    /// Creates a new [RunnerCtx] from an embedded runner handle.
    pub fn embedded(handle: TestSequencer) -> Self {
        Self::Embedded(handle)
    }

    /// Returns a handle to the binary runner if it is a binary runner.
    pub fn as_binary(&self) -> Option<&KatanaRunner> {
        match self {
            Self::Binary(handle) => Some(handle),
            _ => None,
        }
    }

    /// Returns a handle to the embedded runner if it is an embedded runner.
    pub fn as_embedded(&self) -> Option<&TestSequencer> {
        match self {
            Self::Embedded(handle) => Some(handle),
            _ => None,
        }
    }
}

impl From<KatanaRunner> for RunnerCtx {
    fn from(runner: KatanaRunner) -> Self {
        RunnerCtx::Binary(runner)
    }
}

impl From<TestSequencer> for RunnerCtx {
    fn from(sequencer: TestSequencer) -> Self {
        RunnerCtx::Embedded(sequencer)
    }
}
