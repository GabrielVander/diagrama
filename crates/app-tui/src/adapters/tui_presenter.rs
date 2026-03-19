use std::sync::{Arc, Mutex, MutexGuard};

use lib_core::use_cases::load_graph::LoadGraphUseCase;

pub trait TuiPresenter {
    fn state(&self) -> TuiEvent;

    fn load_graph(self: Arc<Self>, source: String) -> smol::Task<()>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum TuiEvent {
    Initial,
    LoadingGraph,
    Error(String),
    PreviewReady(String),
}

pub struct TuiPresenterImpl<T: LoadGraphUseCase + Sync + Send + 'static> {
    load_graph_use_case: Arc<T>,
    last_emitted_event: Mutex<TuiEvent>,
}

impl<T: LoadGraphUseCase + Sync + Send + 'static> TuiPresenterImpl<T> {
    pub fn new(load_graph_use_case: Arc<T>) -> Self {
        Self {
            load_graph_use_case,
            last_emitted_event: Mutex::new(TuiEvent::Initial),
        }
    }

    fn emit(&self, event: TuiEvent) {
        let mut guard: MutexGuard<TuiEvent> = self.last_emitted_event.lock().unwrap();
        *guard = event;
    }
}

impl<T: LoadGraphUseCase + Sync + Send + 'static> TuiPresenter for TuiPresenterImpl<T> {
    fn state(&self) -> TuiEvent {
        self.last_emitted_event.lock().unwrap().clone()
    }

    fn load_graph(self: Arc<Self>, source: String) -> smol::Task<()> {
        let self_clone: Arc<Self> = self.clone();
        let use_case: Arc<T> = self.load_graph_use_case.clone();

        self.emit(TuiEvent::LoadingGraph);

        smol::spawn(async move {
            let _ = use_case
                .execute(source.as_str())
                .await
                .inspect(|graph| self_clone.emit(TuiEvent::PreviewReady(format!("{:?}", graph))))
                .inspect_err(|e| self_clone.emit(TuiEvent::Error(e.clone())));
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use async_trait::async_trait;
    use lib_core::{entities::graph::Graph, use_cases::load_graph::LoadGraphUseCase};

    use crate::adapters::tui_presenter::{TuiEvent, TuiPresenter, TuiPresenterImpl};

    #[test]
    fn initial_event() {
        let load_graph: Arc<FakeLoadGraph> = Arc::new(FakeLoadGraph {
            result: Err("Not implemented".to_owned()),
        });
        let presenter: TuiPresenterImpl<FakeLoadGraph> =
            TuiPresenterImpl::<FakeLoadGraph>::new(load_graph.clone());

        assert_eq!(presenter.state(), TuiEvent::Initial)
    }

    #[test]
    fn load_graph_should_emit_loading_event() {
        smol::block_on(async {
            let load_graph: Arc<FakeLoadGraph> = Arc::new(FakeLoadGraph {
                result: Err("Not implemented".to_owned()),
            });
            let presenter: Arc<TuiPresenterImpl<FakeLoadGraph>> =
                Arc::new(TuiPresenterImpl::<FakeLoadGraph>::new(load_graph.clone()));

            let task: smol::Task<()> = presenter.clone().load_graph("source_code".to_owned());

            assert_eq!(presenter.state(), TuiEvent::LoadingGraph);

            std::mem::drop(task);
        });
    }

    #[test]
    fn load_graph_should_emit_error_event_on_failure() {
        smol::block_on(async {
            let load_graph: Arc<FakeLoadGraph> = Arc::new(FakeLoadGraph {
                result: Err("Not implemented".to_owned()),
            });
            let presenter: Arc<TuiPresenterImpl<FakeLoadGraph>> =
                Arc::new(TuiPresenterImpl::<FakeLoadGraph>::new(load_graph.clone()));

            presenter.clone().load_graph("source_code".to_owned()).await;

            assert_eq!(
                presenter.state(),
                TuiEvent::Error("Not implemented".to_owned())
            );
        });
    }

    #[test]
    fn load_graph_should_emit_parsed_graph_on_success() {
        smol::block_on(async {
            let load_graph: Arc<FakeLoadGraph> = Arc::new(FakeLoadGraph {
                result: Ok(Graph::default()),
            });
            let presenter: Arc<TuiPresenterImpl<FakeLoadGraph>> =
                Arc::new(TuiPresenterImpl::<FakeLoadGraph>::new(load_graph.clone()));

            presenter.clone().load_graph("source_code".to_owned()).await;

            assert_eq!(
                presenter.state(),
                TuiEvent::PreviewReady(format!("{:?}", Graph::default()))
            );
        });
    }

    #[test]
    fn last_event_should_be_returned_if_no_new_events_were_emitted() {
        smol::block_on(async {
            let load_graph: Arc<FakeLoadGraph> = Arc::new(FakeLoadGraph {
                result: Err("Some error".to_owned()),
            });

            let presenter: Arc<TuiPresenterImpl<FakeLoadGraph>> =
                Arc::new(TuiPresenterImpl::<FakeLoadGraph>::new(load_graph.clone()));

            assert_eq!(presenter.state(), TuiEvent::Initial);
            assert_eq!(presenter.state(), TuiEvent::Initial);

            let task: smol::Task<()> = presenter.clone().load_graph("source_code".to_owned());

            assert_eq!(presenter.state(), TuiEvent::LoadingGraph);

            assert_eq!(presenter.state(), TuiEvent::LoadingGraph);

            task.await;

            assert_eq!(presenter.state(), TuiEvent::Error("Some error".to_owned()));

            smol::future::yield_now().await;
        });
    }

    struct FakeLoadGraph {
        result: Result<Graph, String>,
    }

    #[async_trait]
    impl LoadGraphUseCase for FakeLoadGraph {
        async fn execute(&self, _: &str) -> Result<Graph, String> {
            self.result.clone()
        }
    }
}
