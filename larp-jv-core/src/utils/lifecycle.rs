//! Lifecycle utilities for async tasks

use crate::error::LarpResult;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{mpsc, oneshot, Notify};
use tokio::time::{Duration, Instant};

/// Lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Created,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

/// Lifecycle manager for async services
pub struct LifecycleManager {
    state: Arc<tokio::sync::RwLock<LifecycleState>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    shutdown_rx: Option<mpsc::Receiver<()>>,
    notify: Arc<Notify>,
}

impl LifecycleManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1);
        Self {
            state: Arc::new(tokio::sync::RwLock::new(LifecycleState::Created)),
            shutdown_tx: Some(tx),
            shutdown_rx: Some(rx),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn state(&self) -> LifecycleState {
        *self.state.read().await
    }

    pub async fn set_state(&self, state: LifecycleState) {
        *self.state.write().await = state;
        self.notify.notify_waiters();
    }

    pub async fn wait_for_state(&self, target: LifecycleState) {
        loop {
            if *self.state.read().await == target {
                return;
            }
            self.notify.notified().await;
        }
    }

    pub fn shutdown_signal(&self) -> mpsc::Receiver<()> {
        self.shutdown_rx.clone().expect("shutdown receiver already taken")
    }

    pub async fn shutdown(&self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        self.set_state(LifecycleState::Stopping).await;
    }

    pub async fn wait_shutdown(&self) {
        let mut rx = self.shutdown_signal();
        rx.recv().await;
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Task handle with lifecycle
pub struct TaskHandle<T> {
    handle: tokio::task::JoinHandle<T>,
    lifecycle: Arc<LifecycleManager>,
}

impl<T> TaskHandle<T> {
    pub fn new<F, Fut>(lifecycle: Arc<LifecycleManager>, fut: F) -> Self
    where
        F: FnOnce(Arc<LifecycleManager>) -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let lc = lifecycle.clone();
        let handle = tokio::spawn(async move {
            lc.set_state(LifecycleState::Running).await;
            let result = fut(lc).await;
            lifecycle.set_state(LifecycleState::Stopped).await;
            result
        });
        Self { handle, lifecycle }
    }

    pub async fn await_result(self) -> LarpResult<T> {
        self.handle.await.map_err(|e| crate::error::LarpError::Other(e.to_string()))
    }

    pub fn abort(&self) {
        self.handle.abort();
        self.lifecycle.shutdown().await;
    }
}

/// Run with timeout
pub async fn with_timeout<F, T>(duration: Duration, fut: F) -> LarpResult<T>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, fut)
        .await
        .map_err(|_| crate::error::LarpError::Other("timeout".to_string()))
}

/// Run with retry
pub async fn with_retry<F, T, E>(mut fut: F, max_attempts: usize, base_delay: Duration) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Debug,
{
    let mut delay = base_delay;
    for attempt in 0..max_attempts {
        match fut() {
            Ok(v) => return Ok(v),
            Err(e) => {
                if attempt == max_attempts - 1 {
                    return Err(e);
                }
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }
    }
    unreachable!()
}

/// Async task group (like errgroup in Go)
pub struct TaskGroup {
    tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl TaskGroup {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn spawn<F, Fut>(&mut self, fut: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.tasks.push(tokio::spawn(fut()));
    }

    pub async fn wait_all(self) -> LarpResult<()> {
        for task in self.tasks {
            task.await.map_err(|e| crate::error::LarpError::Other(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn wait_any(self) -> LarpResult<()> {
        for task in self.tasks {
            if let Ok(result) = task.await {
                result?;
            }
        }
        Ok(())
    }
}

impl Default for TaskGroup {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_lifecycle_manager() {
        let lc = LifecycleManager::new();
        assert_eq!(lc.state().await, LifecycleState::Created);
        lc.set_state(LifecycleState::Starting).await;
        assert_eq!(lc.state().await, LifecycleState::Starting);
    }

    #[tokio::test]
    async fn test_with_timeout() {
        let result = with_timeout(Duration::from_millis(100), async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            42
        }).await;
        assert_eq!(result.unwrap(), 42);

        let result = with_timeout(Duration::from_millis(50), async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            42
        }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_task_group() {
        let mut group = TaskGroup::new();
        group.spawn(|| async { tokio::time::sleep(Duration::from_millis(10)).await; });
        group.spawn(|| async { tokio::time::sleep(Duration::from_millis(20)).await; });
        group.wait_all().await.unwrap();
    }
}
