use chashmap::{CHashMap, ReadGuard};
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
    resource: RefCell<Option<String>>,
    version: RefCell<Option<HttpVersion>>,
    headers: CHashMap<String, String>,
    headers_finished: bool,
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
    #[inline(always)]
    async fn next(&mut self) -> Option<u8> {
        if self.finished {
            return None;
        }
        if self.cursor >= self.max_cursor {
            let mut buff = [0u8; 1024];
            let n = match self.stream.read(&mut buff).await {
                Ok(x) => x,
                _ => 0,
            };
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

macro_rules! add_part {
    (
        Name: $name:tt,
        Type: $ret:ident,
        Before: $before: tt,
        Parser: |$stream: ident| $parser: expr,
    ) => {
        pub async fn $name(&self) -> Ref<'_, $ret> {
            let inner = self.inner.$name.borrow();
            if inner.is_some() {
                return Ref::map(inner, |x| x.as_ref().unwrap());
            }
            drop(inner);
            add_part!(@check-before self, $before);
            #[allow(unused_mut)]
            let mut $stream = self.stream.borrow_mut();
            let result = $parser;
            let result = result.await;
            drop($stream);
            {
                let inner = &mut self.inner.$name.borrow_mut();
                **inner = Some(result);
                // decrement the mut count
            }
            let inner = self.inner.$name.borrow();
            return Ref::map(inner, |x| x.as_ref().unwrap());
        }
    };
    (@check-before $rec:ident, None) => {};
    (@check-before $rec:ident, $before: ident) => {{
        let no_before = $rec.inner.$before.borrow().is_none();
        if no_before {
            drop($rec.$before().await);
        }
    }}
}

impl HttpLazyStreamReader {
    pub fn new(stream: Pin<Box<dyn AsyncRead>>) -> Self {
        let stream_reader = AsyncReadStream::new(stream);
        Self {
            stream: RefCell::new(stream_reader),
            inner: Inner::default(),
        }
    }

    add_part!(
        Name: method,
        Type: HttpMethod,
        Before: None,
        Parser: |stream| async {
            let mut method = Vec::with_capacity(10);
            while let Some(b) = stream.next().await {
                match b {
                    0 => panic!(""),
                    b' ' => break,
                    x => method.push(x),
                }
            }
            match &method[..] {
                b"GET" => HttpMethod::Get,
                b"POST" => HttpMethod::Post,
                b"PUT" => HttpMethod::Put,
                b"DELETE" => HttpMethod::Delete,
                b"HEAD" => HttpMethod::Head,
                _ => panic!(""),
            }
        },
    );

    add_part!(
        Name: resource,
        Type: String,
        Before: method,
        Parser: |stream| async {
            let mut resource = Vec::with_capacity(10);
            while let Some(b) = stream.next().await {
                match b {
                    0 => panic!(""),
                    b' ' => break,
                    x => resource.push(x),
                }
            }
            String::from_utf8(resource).unwrap()
        },
    );

    add_part!(
        Name: version,
        Type: HttpVersion,
        Before: resource,
        Parser: |stream| async {
            let mut version = Vec::with_capacity(10);
            while let Some(b) = stream.next().await {
                match b {
                    0 => panic!(""),
                    b'\n' => break,
                    x => version.push(x),
                }
            }
            match &version[..] {
                b"HTTP/0.9" => HttpVersion::Http0_9,
                b"HTTP/1.0" => HttpVersion::Http1_0,
                b"HTTP/1.1" => HttpVersion::Http1_1,
                _ => panic!("{:?} is not supported", &version),
            }
        },
    );

    async fn header_inner<'a>(
        &'a self,
        name: &str,
        get_all: bool,
    ) -> Option<ReadGuard<'a, String, String>> {
        {
            /* return value if pressent */
            let val = self.inner.headers.get(name);
            if val.is_some() {
                return val;
            }
        };
        if self.inner.headers_finished {
            return None;
        }
        // we should've parsed until http version
        add_part!(@check-before self, version);

        let mut stream = self.stream.borrow_mut();
        let mut has_more_headers = true;
        'outer: loop {
            let mut header_name = Vec::with_capacity(10);
            let mut header_value = Vec::with_capacity(10);
            let mut is_name = true;
            let mut is_start = true;
            'inner: while let Some(b) = stream.next().await {
                let container = if is_name {
                    &mut header_name
                } else {
                    &mut header_value
                };
                match b {
                    0 => panic!(""),
                    b'\n' => {
                        if is_start {
                            has_more_headers = false;
                        } else {
                            // header row is finished
                            break 'inner;
                        }
                    }
                    b'\r' => {
                        // header is finished
                        break 'outer;
                    }
                    x => {
                        if !has_more_headers {
                            panic!("expected '\\r' found '{}'", x);
                        }
                        is_start = false;
                        if x != b':' {
                            container.push(x);
                        } else {
                            is_name = false;
                        }
                    }
                };
            }
            let header_name = String::from_utf8(header_name).unwrap();
            let found = !get_all && header_name == name;
            let header_value = String::from_utf8(header_value).unwrap();
            self.inner.headers.insert(header_name, header_value);
            if found {
                let val = self.inner.headers.get(name);
                return val;
            }
        }

        None
    }

    pub async fn header<'a>(&'a self, name: &str) -> Option<ReadGuard<'a, String, String>> {
        self.header_inner(name, false).await
    }

    // TODO: add body support
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

    macro_rules! test {
        (@test $m: ident, ($r_name:ident, $r:literal)) => {
            paste! {
            #[tokio::test]
            async fn [<test_first_method_ $m:lower _resource_ $r_name:lower>]() {
                let (m_str, m_enum) = (
                    stringify!($m),
                    HttpMethod::[<$m:camel>]
                );
                let expected_resource = $r;
                let payload = format!("{} {} HTTP/1.1", m_str, expected_resource);
                let payload = payload.as_bytes();
                let mock_read = MockRead(payload.to_vec());
                let stream = Box::pin(mock_read);
                let reader = HttpLazyStreamReader::new(stream);
                // read method
                let method = reader.method().await;
                assert_eq!(m_enum, *method);
                let method = reader.method().await;
                assert_eq!(m_enum, *method);
                // read resource
                let resource = reader.resource().await;
                assert_eq!(*resource, *expected_resource);
                let resource = reader.resource().await;
                assert_eq!(*resource, *expected_resource);
            }

            #[tokio::test]
            async fn [<test_first_resource_ $r_name:lower _method_ $m:lower >]() {
                let (m_str, m_enum) = (
                    stringify!($m),
                    HttpMethod::[<$m:camel>]
                );
                let expected_resource = $r;
                let payload = format!("{} {} HTTP/1.1", m_str, expected_resource);
                let payload = payload.as_bytes();
                let mock_read = MockRead(payload.to_vec());
                let stream = Box::pin(mock_read);
                let reader = HttpLazyStreamReader::new(stream);
                // read resource
                let resource = reader.resource().await;
                assert_eq!(*resource, *expected_resource);
                let resource = reader.resource().await;
                assert_eq!(*resource, *expected_resource);
                // read method
                let method = reader.method().await;
                assert_eq!(m_enum, *method);
                let method = reader.method().await;
                assert_eq!(m_enum, *method);
            }
        }};
        (@test-resource ($r_name:ident, $r:literal), ($($m:ident),*)) => {
            $(
                test!(@test $m, ($r_name, $r));
            )*
        };
        ({
            Resources: $(($r_name:ident, $r:literal))*,
            Methods: $m:tt,
        }) => {
            $(
                test!(@test-resource ($r_name, $r), $m);
            )*
        }
    }

    test!({
        Resources: (home, "/home") (root, "/"),
        Methods: (GET, POST, PUT, DELETE, HEAD),
    });

    // TODO: add support for more type of tests
    // TODO: add multi thread tests
    // TODO: add support for incomplete streams
}
