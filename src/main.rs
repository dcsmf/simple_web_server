use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use simple_web_server::ThreadPool;

fn main() {
    //创建tcp监听
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    //创建线程池，最大线程为4
    let pool = ThreadPool::new(4);//线程池会有一个new函数通过传参限制线程数

    for stream in listener.incoming().take(2) {//这里的take函数表示只循环两次就break了
        let stream = stream.unwrap();

        pool.execute(|| {//线程池会有execute方法来传递需要执行的代码
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];//缓冲区
    stream.read(&mut buffer).unwrap();//把得到的http请求写入缓冲区

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {//如果是“/”请求
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();//给客户端返回响应
    stream.flush().unwrap();//flush方法阻塞程序直到响应写完
}