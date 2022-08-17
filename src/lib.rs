use std::sync::{Arc, mpsc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;//trait对象的类型别名

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,//用来存储线程
}

enum Message {
    NewJob(Job),
    //具体的工作消息
    Terminate,//停机消息，让线程自行销毁
}

impl ThreadPool {
    /// 创建线程池。
    ///
    /// 线程池中线程的数量。
    ///
    /// # Panics
    ///
    /// `new` 函数在 size 为 0 时会 panic。
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();//构建线程间通信，为了发送工作或停机消息

        let receiver = Arc::new(Mutex::new(receiver));//把receiver包裹起来，以便多所有者，但一次只能一个线程用

        let mut workers = Vec::with_capacity(size);//存储线程的集合

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));//创建多个Worker实例，每个实例都包含一个线程
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);//把得到的闭包封装一下
        self.sender.send(Message::NewJob(job)).unwrap();//发送给接收端
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        //创建一个无限循环的线程来等待工作的到来
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();//一直等待接收
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} 得到了一个工作，正在执行", id);
                    job();//执行得到的闭包
                }
                Message::Terminate => {
                    break;
                }
            }
        });
        Worker { id, thread: Some(thread) }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("向所有线程发送关闭命令");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("正在关闭Worker {}", worker.id);

            //因为join需要获取thread的所有权，所以用take把Option的Some(thread)提取出Worker实例
            //如果是None，则不会执行这代码
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}