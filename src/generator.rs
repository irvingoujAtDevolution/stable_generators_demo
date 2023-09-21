use std::future::Future;
use std::pin::Pin;
use std::process::Output;
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
    pub fn suspend(&self, value: YieldTy) -> Interrupt<YieldTy, ResumeTy> {
        Interrupt {
            value_to_yield: Some(value),
            yielded_value: Arc::clone(&self.yielded_value),
            resumed_value: Arc::clone(&self.resumed_value),
            ready_to_resume: false,
        }
    }

    pub fn new()->Self{
        let yielded_value = Arc::new(Mutex::new(None));
        let resumed_value = Arc::new(Mutex::new(None));

        let yield_point = YieldPoint {
            yielded_value: Arc::clone(&yielded_value),
            resumed_value: Arc::clone(&resumed_value),
        };

        yield_point
    }
}

impl<YieldTy, ResumeTy> Clone for YieldPoint<YieldTy, ResumeTy> {
    fn clone(&self) -> Self {
        Self {
            yielded_value: self.yielded_value.clone(),
            resumed_value: self.resumed_value.clone(),
        }
    }
}


type YieldedValue<T> = Arc<Mutex<Option<T>>>;

type ResumedValue<T> = Arc<Mutex<Option<T>>>;

type PinnedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub enum GeneratorState<YieldTy, OutTy> {
    Suspended(YieldTy),
    Completed(OutTy),
}

pub struct Generator<'a, YieldTy, ResumeTy, OutTy> {
    yielded_value: YieldedValue<YieldTy>,
    resumed_value: ResumedValue<ResumeTy>,
    task: Option<PinnedFuture<'a, OutTy>>,
}

impl<'a, YieldTy, ResumeTy, OutTy> Generator<'a, YieldTy, ResumeTy, OutTy>
where
    OutTy: Send + Sync + 'a,
{
    pub fn new<Producer, Task>(producer: Producer) -> Self
    where
        Producer: FnOnce(YieldPoint<YieldTy, ResumeTy>) -> Task,
        Task: Future<Output = OutTy> + Send + 'a,
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
            task: Some(Box::pin(producer(yield_point))),
        }
    }



    pub fn new_with_yield_point<Task>(yield_point: YieldPoint<YieldTy, ResumeTy>,task:Task) -> Self
    where
        // Producer: FnOnce(YieldPoint<YieldTy, ResumeTy>) -> Task,
        Task: Future<Output = OutTy> + Send + 'a,
    {
        let yielded_value = yield_point.yielded_value.clone();
        let resumed_value = yield_point.resumed_value.clone();

        Self {
            yielded_value : yielded_value,
            resumed_value : resumed_value,
            task: Some(Box::pin(task)),
        }
    }

    
    pub fn new_empty() -> Self{
        return Self{
            resumed_value:Arc::new(Mutex::new(None)),
            yielded_value:Arc::new(Mutex::new(None)),
            task:None
        };
    }
        
    pub fn accept_task<Task>(&mut self,task:Task)
    where  Task: Future<Output = OutTy> + Send + 'a,
    {
        self.task = Some(Box::pin(task))
    }

    pub fn accept_yield_point(&mut self,yield_point:&YieldPoint<YieldTy, ResumeTy>){
        let clone = yield_point.clone();
        self.yielded_value = clone.yielded_value;
        self.resumed_value = clone.resumed_value;
    }

    pub fn start(&mut self) -> GeneratorState<YieldTy, OutTy> {
        self.step()
    }

    pub fn resume(&mut self, value: ResumeTy) -> GeneratorState<YieldTy, OutTy> {
        *self.resumed_value.try_lock().unwrap() = Some(value);
        self.step()
    }

    fn step(&mut self) -> GeneratorState<YieldTy, OutTy> {
        match execute_one_step(self.task.as_mut().expect("no task to execute")) {
            None => {
                let value = self.yielded_value.try_lock().unwrap().take().unwrap();
                GeneratorState::Suspended(value)
            }
            Some(value) => GeneratorState::Completed(value),
        }
    }
}

fn execute_one_step<OutTy>(task: &mut PinnedFuture<OutTy>) -> Option<OutTy> {
    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: std::sync::Arc<Self>) {
            // do nothing
        }
    }

    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);

    match task.as_mut().poll(&mut context) {
        Poll::Pending => None,
        Poll::Ready(item) => Some(item),
    }
}

impl<'a, YieldTy, ResumeTy, OutTy> Future for Generator<'a, YieldTy, ResumeTy, OutTy> {
    type Output = OutTy;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match execute_one_step(self.task.as_mut().unwrap()) {
            Some(res) => Poll::Ready(res),
            None => Poll::Pending,
        }
    }
}