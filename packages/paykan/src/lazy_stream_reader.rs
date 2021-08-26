use std::{
    cell::{Ref, RefCell},
    pin::Pin,
};
use tokio::io::{AsyncRead, AsyncReadExt};

/*

GET /hello.htm HTTP/1.1
User-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)
Host: www.tutorialspoint.com
Accept-Language: en-us
Accept-Encoding: gzip, deflate
Connection: Keep-Alive

*/

#[derive(PartialEq, Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Delete,
    Put,
    Head,
}

pub enum HttpVersion {
    Http0_9 = 09,
    Http1_0 = 10,
    Http1_1 = 11,
    Http2_0 = 20,
}

#[derive(Default)]
struct Inner {
    method: RefCell<Option<HttpMethod>>,
    _resource: RefCell<Option<String>>,
    _version: RefCell<Option<HttpVersion>>,
}

pub struct HttpLazyStreamReader {
    stream: RefCell<AsyncReadStream>,
    inner: Inner,
}

struct AsyncReadStream {
    stream: Pin<Box<dyn AsyncRead>>,
    buff: [u8; 1024],
    cursor: usize,
    max_cursor: usize,
    finished: bool,
}

impl AsyncReadStream {
    pub fn new(stream: Pin<Box<dyn AsyncRead>>) -> Self {
        Self {
            buff: [0u8; 1024],
            stream,
            cursor: 0,
            max_cursor: 0,
            finished: false,
        }
    }
}

impl AsyncReadStream {
    async fn next(&mut self) -> Option<u8> {
        if self.finished {
            return None;
        }
        if self.cursor >= self.max_cursor {
            let mut buff = [0u8; 1024];
            let n = self.stream.read(&mut buff).await.ok()?;
            if n == 0 {
                self.finished = true;
                return None;
            }
            self.buff = buff;
            let item = self.buff[0];
            self.cursor = 1;
            self.max_cursor = n - 1;
            return Some(item);
        }
        let item = self.buff[self.cursor];
        self.cursor += 1;
        return Some(item);
    }
}

impl HttpLazyStreamReader {
    pub fn new(stream: Pin<Box<dyn AsyncRead>>) -> Self {
        let stream_reader = AsyncReadStream::new(stream);
        Self {
            stream: RefCell::new(stream_reader),
            inner: Inner::default(),
        }
    }

    pub async fn method(&self) -> Ref<'_, HttpMethod> {
        let inner_method = self.inner.method.borrow();
        if inner_method.is_some() {
            return Ref::map(inner_method, |x| x.as_ref().unwrap());
        }
        let mut method = Vec::with_capacity(10);
        {
            let mut stream = self.stream.borrow_mut();
            'outer: while let Some(b) = stream.next().await {
                match b {
                    0 => panic!(""),
                    b' ' => break 'outer,
                    x => method.push(x),
                }
            }
        }

        let result = match &method[..] {
            b"GET" => HttpMethod::Get,
            b"POST" => HttpMethod::Post,
            b"PUT" => HttpMethod::Put,
            b"DELETE" => HttpMethod::Delete,
            b"HEAD" => HttpMethod::Head,
            _ => panic!(""),
        };
        drop(inner_method);
        {
            let inner_method = &mut self.inner.method.borrow_mut();
            **inner_method = Some(result);
            // decrement the mut count
        }
        let inner_method = self.inner.method.borrow();
        return Ref::map(inner_method, |x| x.as_ref().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paste::paste;
    use std::task::Poll;

    struct MockRead(Vec<u8>);
    impl AsyncRead for MockRead {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let internal_len = self.0.len();
            let result: Vec<_> = self.0.drain(0..internal_len.min(buf.capacity())).collect();
            buf.put_slice(&result[..]);
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn test_method_can_pattern_match() {
        let payload = format!("{} /hello.htm HTTP/1.1", "GET");
        let payload = payload.as_bytes();
        let mock_read = MockRead(payload.to_vec());
        let reader = HttpLazyStreamReader::new(Box::pin(mock_read));
        let result = reader.method().await;
        if let HttpMethod::Get = *result {
        } else {
            assert!(false, "should match");
        }
    }

    macro_rules! ident_to_method {
        ($m: ident) => {
            paste! { HttpMethod::[<$m:camel>] }
        };
    }

    macro_rules! test_method {
        ($m: ident) => {
            paste! {
            #[tokio::test]
            async fn [<test_method_ $m:lower>]() {
                let (m_str, m_enum) = (
                    stringify!($m),
                    ident_to_method!($m)
                );
                let payload = format!("{} /hello.htm HTTP/1.1", m_str);
                let payload = payload.as_bytes();
                let mock_read = MockRead(payload.to_vec());
                let stream = Box::pin(mock_read);
                let reader = HttpLazyStreamReader::new(stream);
                let method = reader.method().await;
                assert_eq!(m_enum, *method);
                let method = reader.method().await;
                assert_eq!(m_enum, *method);
            }
        }};
        ($($m:ident)* ) => {
            $(
                test_method!($m);
            )*
        }
    }

    test_method!(POST);
    test_method!(PUT);
    test_method!(DELETE);
    test_method!(HEAD);
    test_method!(GET);
}
