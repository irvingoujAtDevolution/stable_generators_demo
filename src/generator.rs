use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

// This is the only future that our minimal async executor will ever handle in order to emulate generators

pub struct Interrupt<YieldTy, ResumeTy> {
    value_to_yield: Option<YieldTy>,
    yielded_value: YieldedValue<YieldTy>,
    resumed_value: ResumedValue<ResumeTy>,
    ready_to_resume: bool,
}

impl<YieldTy, ResumeTy> Future for Interrupt<YieldTy, ResumeTy>
where
    YieldTy: Unpin,
{
    type Output = ResumeTy;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if this.ready_to_resume {
            let resumed_value = this.resumed_value.try_lock().unwrap().take().unwrap();
            Poll::Ready(resumed_value)
        } else {
            let value_to_yield = this.value_to_yield.take().unwrap();
            *this.yielded_value.try_lock().unwrap() = Some(value_to_yield);
            this.ready_to_resume = true;
            Poll::Pending
        }
    }
}

pub struct YieldPoint<YieldTy, ResumeTy> {
    yielded_value: YieldedValue<YieldTy>,
    resumed_value: ResumedValue<ResumeTy>,
}

impl<YieldTy, ResumeTy> YieldPoint<YieldTy, ResumeTy> {
    pub fn suspend(&mut self, value: YieldTy) -> Interrupt<YieldTy, ResumeTy> {
        Interrupt {
            value_to_yield: Some(value),
            yielded_value: Arc::clone(&self.yielded_value),
            resumed_value: Arc::clone(&self.resumed_value),
            ready_to_resume: false,
        }
    }
}

type YieldedValue<T> = Arc<Mutex<Option<T>>>;

type ResumedValue<T> = Arc<Mutex<Option<T>>>;

type PinnedFuture<T> = Pin<Box<dyn Future<Output = T> + Send + Sync>>;

pub enum GeneratorState<YieldTy, OutTy> {
    Suspended(YieldTy),
    Completed(OutTy),
}

pub struct Generator<YieldTy, ResumeTy, OutTy> {
    yielded_value: YieldedValue<YieldTy>,
    resumed_value: ResumedValue<ResumeTy>,
    generator: PinnedFuture<OutTy>,
}

impl<YieldTy, ResumeTy, OutTy> Generator<YieldTy, ResumeTy, OutTy> {
    pub fn new<Producer, Generator>(producer: Producer) -> Self
    where
        Producer: FnOnce(YieldPoint<YieldTy, ResumeTy>) -> Generator,
        Generator: Future<Output = OutTy> + Send + Sync + 'static,
    {
        let yielded_value = Arc::new(Mutex::new(None));
        let resumed_value = Arc::new(Mutex::new(None));

        let yield_point = YieldPoint {
            yielded_value: Arc::clone(&yielded_value),
            resumed_value: Arc::clone(&resumed_value),
        };

        Self {
            yielded_value,
            resumed_value,
            generator: Box::pin(producer(yield_point)),
        }
    }

    pub fn start(&mut self) -> GeneratorState<YieldTy, OutTy> {
        self.step()
    }

    pub fn resume(&mut self, value: ResumeTy) -> GeneratorState<YieldTy, OutTy> {
        *self.resumed_value.try_lock().unwrap() = Some(value);
        self.step()
    }

    fn step(&mut self) -> GeneratorState<YieldTy, OutTy> {
        match execute_one_step(&mut self.generator) {
            None => {
                let value = self.yielded_value.try_lock().unwrap().take().unwrap();
                GeneratorState::Suspended(value)
            }
            Some(value) => GeneratorState::Completed(value),
        }
    }
}

fn execute_one_step<OutTy>(generator: &mut PinnedFuture<OutTy>) -> Option<OutTy> {
    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: std::sync::Arc<Self>) {
            // do nothing
        }
    }

    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);

    match generator.as_mut().poll(&mut context) {
        Poll::Pending => None,
        Poll::Ready(item) => Some(item),
    }
}
