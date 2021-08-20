use std::{hint::unreachable_unchecked, pin::Pin};

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
    method: Option<HttpMethod>,
    resource: Option<String>,
    version: Option<HttpVersion>,
}

pub struct HttpLazyStreamReader {
    stream: Pin<Box<dyn AsyncRead>>,
    inner: Inner,
    buffer: Vec<u8>,
}

impl HttpLazyStreamReader {
    pub fn new(stream: Pin<Box<dyn AsyncRead>>) -> Self {
        Self {
            stream,
            inner: Inner::default(),
            buffer: Vec::new(),
        }
    }

    pub async fn method(&mut self) -> &HttpMethod {
        let inner_method = &mut self.inner.method;
        if let Some(x) = inner_method {
            return x;
        }
        let mut method = Vec::with_capacity(10);
        loop {
            let mut buf = [0u8; 1];
            self.stream.read(&mut buf).await.unwrap();
            match buf[0] {
                0 => panic!(""),
                b' ' => break,
                x => method.push(x),
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

        *inner_method = Some(result);

        match inner_method {
            Some(x) => x,
            _ => unsafe {
                unreachable_unchecked();
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paste::paste;
    use std::{ptr::addr_of_mut, task::Poll};

    struct MockRead(Vec<u8>);
    impl AsyncRead for MockRead {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let result: Vec<_> = self.0.drain(0..buf.capacity()).collect();
            buf.put_slice(&result[..]);
            Poll::Ready(Ok(()))
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
                let mut reader = HttpLazyStreamReader::new(Box::pin(mock_read));
                let mock_read = unsafe { &*(addr_of_mut!(reader.stream) as *mut Box<MockRead>) };
                let method_size = m_str.len() + 1;
                assert!(&mock_read.0.len() == &payload.len());
                let method = reader.method().await;
                assert!(&mock_read.0.len() == &(payload.len() - method_size));
                assert_eq!(&m_enum, method);
                let method = reader.method().await;
                assert!(&mock_read.0.len() == &(payload.len() - method_size));
                assert_eq!(&m_enum, method);
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
