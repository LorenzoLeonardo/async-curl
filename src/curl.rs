use std::{path::Path, time::Duration};

use curl::easy::{
    Auth, Easy2, Form, Handler, HttpVersion, IpResolve, List, NetRc, ProxyType, SslOpt, SslVersion,
    TimeCondition,
};

use crate::{actor::CurlActor, error::Error};

/// A type-state struct in building the HttpClient.
pub struct Build;
/// A type-state struct in building the HttpClient.
pub struct Perform;

/// The HTTP Client struct that wraps curl Easy2.
pub struct HttpClient<C, S>
where
    C: Handler + std::fmt::Debug + Send + 'static,
{
    curl: CurlActor<C>,
    easy: Easy2<C>,
    _state: S,
}

impl<C> HttpClient<C, Build>
where
    C: Handler + std::fmt::Debug + Send + 'static,
{
    /// Creates a new HTTP Client.
    ///
    /// The [`CurlActor`](https://docs.rs/async-curl/latest/async_curl/actor/struct.CurlActor.html) is the actor handler that can be cloned to be able to handle multiple request sender
    /// and a single consumer that is spawned in the background upon creation of this object to be able to achieve
    /// non-blocking I/O during curl perform.
    pub fn new(curl: CurlActor<C>, collector: C) -> Self {
        Self {
            curl,
            easy: Easy2::new(collector),
            _state: Build,
        }
    }

    // =========================================================================
    // Behavior options

    /// Configures this handle to have verbose output to help debug protocol
    /// information.
    ///
    /// By default output goes to stderr, but the `stderr` function on this type
    /// can configure that. You can also use the `debug_function` method to get
    /// all protocol data sent and received.
    ///
    /// By default, this option is `false`.
    pub fn verbose(mut self, verbose: bool) -> Result<Self, Error<C>> {
        self.easy.verbose(verbose).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Indicates whether header information is streamed to the output body of
    /// this request.
    ///
    /// This option is only relevant for protocols which have header metadata
    /// (like http or ftp). It's not generally possible to extract headers
    /// from the body if using this method, that use case should be intended for
    /// the `header_function` method.
    ///
    /// To set HTTP headers, use the `http_header` method.
    ///
    /// By default, this option is `false` and corresponds to
    /// `CURLOPT_HEADER`.
    pub fn show_header(mut self, show: bool) -> Result<Self, Error<C>> {
        self.easy.show_header(show).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Indicates whether a progress meter will be shown for requests done with
    /// this handle.
    ///
    /// This will also prevent the `progress_function` from being called.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_NOPROGRESS`.
    pub fn progress(mut self, progress: bool) -> Result<Self, Error<C>> {
        self.easy.progress(progress).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Inform libcurl whether or not it should install signal handlers or
    /// attempt to use signals to perform library functions.
    ///
    /// If this option is disabled then timeouts during name resolution will not
    /// work unless libcurl is built against c-ares. Note that enabling this
    /// option, however, may not cause libcurl to work with multiple threads.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_NOSIGNAL`.
    /// Note that this default is **different than libcurl** as it is intended
    /// that this library is threadsafe by default. See the [libcurl docs] for
    /// some more information.
    ///
    /// [libcurl docs]: https://curl.haxx.se/libcurl/c/threadsafe.html
    pub fn signal(mut self, signal: bool) -> Result<Self, Error<C>> {
        self.easy.signal(signal).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Indicates whether multiple files will be transferred based on the file
    /// name pattern.
    ///
    /// The last part of a filename uses fnmatch-like pattern matching.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_WILDCARDMATCH`.
    pub fn wildcard_match(mut self, m: bool) -> Result<Self, Error<C>> {
        self.easy.wildcard_match(m).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Provides the Unix domain socket which this handle will work with.
    ///
    /// The string provided must be a path to a Unix domain socket encoded with
    /// the format:
    ///
    /// ```text
    /// /path/file.sock
    /// ```
    ///
    /// By default this option is not set and corresponds to
    /// [`CURLOPT_UNIX_SOCKET_PATH`](https://curl.haxx.se/libcurl/c/CURLOPT_UNIX_SOCKET_PATH.html).
    pub fn unix_socket(mut self, unix_domain_socket: &str) -> Result<Self, Error<C>> {
        self.easy.unix_socket(unix_domain_socket).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Provides the Unix domain socket which this handle will work with.
    ///
    /// The string provided must be a path to a Unix domain socket encoded with
    /// the format:
    ///
    /// ```text
    /// /path/file.sock
    /// ```
    ///
    /// This function is an alternative to [`Easy2::unix_socket`] that supports
    /// non-UTF-8 paths and also supports disabling Unix sockets by setting the
    /// option to `None`.
    ///
    /// By default this option is not set and corresponds to
    /// [`CURLOPT_UNIX_SOCKET_PATH`](https://curl.haxx.se/libcurl/c/CURLOPT_UNIX_SOCKET_PATH.html).
    pub fn unix_socket_path<P: AsRef<Path>>(mut self, path: Option<P>) -> Result<Self, Error<C>> {
        self.easy.unix_socket_path(path).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // =========================================================================
    // Error options

    // TODO: error buffer and stderr

    /// Indicates whether this library will fail on HTTP response codes >= 400.
    ///
    /// This method is not fail-safe especially when authentication is involved.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_FAILONERROR`.
    pub fn fail_on_error(mut self, fail: bool) -> Result<Self, Error<C>> {
        self.easy.fail_on_error(fail).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // =========================================================================
    // Network options

    /// Provides the URL which this handle will work with.
    ///
    /// The string provided must be URL-encoded with the format:
    ///
    /// ```text
    /// scheme://host:port/path
    /// ```
    ///
    /// The syntax is not validated as part of this function and that is
    /// deferred until later.
    ///
    /// By default this option is not set and `perform` will not work until it
    /// is set. This option corresponds to `CURLOPT_URL`.
    pub fn url(mut self, url: &str) -> Result<Self, Error<C>> {
        self.easy.url(url).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configures the port number to connect to, instead of the one specified
    /// in the URL or the default of the protocol.
    pub fn port(mut self, port: u16) -> Result<Self, Error<C>> {
        self.easy.port(port).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Connect to a specific host and port.
    ///
    /// Each single string should be written using the format
    /// `HOST:PORT:CONNECT-TO-HOST:CONNECT-TO-PORT` where `HOST` is the host of
    /// the request, `PORT` is the port of the request, `CONNECT-TO-HOST` is the
    /// host name to connect to, and `CONNECT-TO-PORT` is the port to connect
    /// to.
    ///
    /// The first string that matches the request's host and port is used.
    ///
    /// By default, this option is empty and corresponds to
    /// [`CURLOPT_CONNECT_TO`](https://curl.haxx.se/libcurl/c/CURLOPT_CONNECT_TO.html).
    pub fn connect_to(mut self, list: List) -> Result<Self, Error<C>> {
        self.easy.connect_to(list).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Indicates whether sequences of `/../` and `/./` will be squashed or not.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_PATH_AS_IS`.
    pub fn path_as_is(mut self, as_is: bool) -> Result<Self, Error<C>> {
        self.easy.path_as_is(as_is).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Provide the URL of a proxy to use.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_PROXY`.
    pub fn proxy(mut self, url: &str) -> Result<Self, Error<C>> {
        self.easy.proxy(url).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Provide port number the proxy is listening on.
    ///
    /// By default this option is not set (the default port for the proxy
    /// protocol is used) and corresponds to `CURLOPT_PROXYPORT`.
    pub fn proxy_port(mut self, port: u16) -> Result<Self, Error<C>> {
        self.easy.proxy_port(port).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set CA certificate to verify peer against for proxy.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXY_CAINFO`.
    pub fn proxy_cainfo(mut self, cainfo: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_cainfo(cainfo).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify a directory holding CA certificates for proxy.
    ///
    /// The specified directory should hold multiple CA certificates to verify
    /// the HTTPS proxy with. If libcurl is built against OpenSSL, the
    /// certificate directory must be prepared using the OpenSSL `c_rehash`
    /// utility.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXY_CAPATH`.
    pub fn proxy_capath<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error<C>> {
        self.easy.proxy_capath(path).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set client certificate for proxy.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXY_SSLCERT`.
    pub fn proxy_sslcert(mut self, sslcert: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_sslcert(sslcert).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify type of the client SSL certificate for HTTPS proxy.
    ///
    /// The string should be the format of your certificate. Supported formats
    /// are "PEM" and "DER", except with Secure Transport. OpenSSL (versions
    /// 0.9.3 and later) and Secure Transport (on iOS 5 or later, or OS X 10.7
    /// or later) also support "P12" for PKCS#12-encoded files.
    ///
    /// By default this option is "PEM" and corresponds to
    /// `CURLOPT_PROXY_SSLCERTTYPE`.
    pub fn proxy_sslcert_type(mut self, kind: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_sslcert_type(kind).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the client certificate for the proxy using an in-memory blob.
    ///
    /// The specified byte buffer should contain the binary content of the
    /// certificate, which will be copied into the handle.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_SSLCERT_BLOB`.
    pub fn proxy_sslcert_blob(mut self, blob: &[u8]) -> Result<Self, Error<C>> {
        self.easy.proxy_sslcert_blob(blob).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set private key for HTTPS proxy.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXY_SSLKEY`.
    pub fn proxy_sslkey(mut self, sslkey: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_sslkey(sslkey).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set type of the private key file for HTTPS proxy.
    ///
    /// The string should be the format of your private key. Supported formats
    /// are "PEM", "DER" and "ENG".
    ///
    /// The format "ENG" enables you to load the private key from a crypto
    /// engine. In this case `ssl_key` is used as an identifier passed to
    /// the engine. You have to set the crypto engine with `ssl_engine`.
    /// "DER" format key file currently does not work because of a bug in
    /// OpenSSL.
    ///
    /// By default this option is "PEM" and corresponds to
    /// `CURLOPT_PROXY_SSLKEYTYPE`.
    pub fn proxy_sslkey_type(mut self, kind: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_sslkey_type(kind).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the private key for the proxy using an in-memory blob.
    ///
    /// The specified byte buffer should contain the binary content of the
    /// private key, which will be copied into the handle.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_SSLKEY_BLOB`.
    pub fn proxy_sslkey_blob(mut self, blob: &[u8]) -> Result<Self, Error<C>> {
        self.easy.proxy_sslkey_blob(blob).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set passphrase to private key for HTTPS proxy.
    ///
    /// This will be used as the password required to use the `ssl_key`.
    /// You never needed a pass phrase to load a certificate but you need one to
    /// load your private key.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_KEYPASSWD`.
    pub fn proxy_key_password(mut self, password: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_key_password(password).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Indicates the type of proxy being used.
    ///
    /// By default this option is `ProxyType::Http` and corresponds to
    /// `CURLOPT_PROXYTYPE`.
    pub fn proxy_type(mut self, kind: ProxyType) -> Result<Self, Error<C>> {
        self.easy.proxy_type(kind).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Provide a list of hosts that should not be proxied to.
    ///
    /// This string is a comma-separated list of hosts which should not use the
    /// proxy specified for connections. A single `*` character is also accepted
    /// as a wildcard for all hosts.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_NOPROXY`.
    pub fn noproxy(mut self, skip: &str) -> Result<Self, Error<C>> {
        self.easy.noproxy(skip).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Inform curl whether it should tunnel all operations through the proxy.
    ///
    /// This essentially means that a `CONNECT` is sent to the proxy for all
    /// outbound requests.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_HTTPPROXYTUNNEL`.
    pub fn http_proxy_tunnel(mut self, tunnel: bool) -> Result<Self, Error<C>> {
        self.easy.http_proxy_tunnel(tunnel).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Tell curl which interface to bind to for an outgoing network interface.
    ///
    /// The interface name, IP address, or host name can be specified here.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_INTERFACE`.
    pub fn interface(mut self, interface: &str) -> Result<Self, Error<C>> {
        self.easy.interface(interface).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Indicate which port should be bound to locally for this connection.
    ///
    /// By default this option is 0 (any port) and corresponds to
    /// `CURLOPT_LOCALPORT`.
    pub fn set_local_port(mut self, port: u16) -> Result<Self, Error<C>> {
        self.easy.set_local_port(port).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Indicates the number of attempts libcurl will perform to find a working
    /// port number.
    ///
    /// By default this option is 1 and corresponds to
    /// `CURLOPT_LOCALPORTRANGE`.
    pub fn local_port_range(mut self, range: u16) -> Result<Self, Error<C>> {
        self.easy.local_port_range(range).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Sets the DNS servers that wil be used.
    ///
    /// Provide a comma separated list, for example: `8.8.8.8,8.8.4.4`.
    ///
    /// By default this option is not set and the OS's DNS resolver is used.
    /// This option can only be used if libcurl is linked against
    /// [c-ares](https://c-ares.haxx.se), otherwise setting it will return
    /// an error.
    pub fn dns_servers(mut self, servers: &str) -> Result<Self, Error<C>> {
        self.easy.dns_servers(servers).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Sets the timeout of how long name resolves will be kept in memory.
    ///
    /// This is distinct from DNS TTL options and is entirely speculative.
    ///
    /// By default this option is 60s and corresponds to
    /// `CURLOPT_DNS_CACHE_TIMEOUT`.
    pub fn dns_cache_timeout(mut self, dur: Duration) -> Result<Self, Error<C>> {
        self.easy.dns_cache_timeout(dur).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Provide the DNS-over-HTTPS URL.
    ///
    /// The parameter must be URL-encoded in the following format:
    /// `https://host:port/path`. It **must** specify a HTTPS URL.
    ///
    /// libcurl does not validate the syntax or use this variable until the
    /// transfer is issued. Even if you set a crazy value here, this method will
    /// still return [`Ok`].
    ///
    /// curl sends `POST` requests to the given DNS-over-HTTPS URL.
    ///
    /// To find the DoH server itself, which might be specified using a name,
    /// libcurl will use the default name lookup function. You can bootstrap
    /// that by providing the address for the DoH server with
    /// [`Easy2::resolve`].
    ///
    /// Disable DoH use again by setting this option to [`None`].
    ///
    /// By default this option is not set and corresponds to `CURLOPT_DOH_URL`.
    pub fn doh_url(mut self, url: Option<&str>) -> Result<Self, Error<C>> {
        self.easy.doh_url(url).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// This option tells curl to verify the authenticity of the DoH
    /// (DNS-over-HTTPS) server's certificate. A value of `true` means curl
    /// verifies; `false` means it does not.
    ///
    /// This option is the DoH equivalent of [`Easy2::ssl_verify_peer`] and only
    /// affects requests to the DoH server.
    ///
    /// When negotiating a TLS or SSL connection, the server sends a certificate
    /// indicating its identity. Curl verifies whether the certificate is
    /// authentic, i.e. that you can trust that the server is who the
    /// certificate says it is. This trust is based on a chain of digital
    /// signatures, rooted in certification authority (CA) certificates you
    /// supply. curl uses a default bundle of CA certificates (the path for that
    /// is determined at build time) and you can specify alternate certificates
    /// with the [`Easy2::cainfo`] option or the [`Easy2::capath`] option.
    ///
    /// When `doh_ssl_verify_peer` is enabled, and the verification fails to
    /// prove that the certificate is authentic, the connection fails. When the
    /// option is zero, the peer certificate verification succeeds regardless.
    ///
    /// Authenticating the certificate is not enough to be sure about the
    /// server. You typically also want to ensure that the server is the server
    /// you mean to be talking to. Use [`Easy2::doh_ssl_verify_host`] for that.
    /// The check that the host name in the certificate is valid for the host
    /// name you are connecting to is done independently of the
    /// `doh_ssl_verify_peer` option.
    ///
    /// **WARNING:** disabling verification of the certificate allows bad guys
    /// to man-in-the-middle the communication without you knowing it. Disabling
    /// verification makes the communication insecure. Just having encryption on
    /// a transfer is not enough as you cannot be sure that you are
    /// communicating with the correct end-point.
    ///
    /// By default this option is set to `true` and corresponds to
    /// `CURLOPT_DOH_SSL_VERIFYPEER`.
    pub fn doh_ssl_verify_peer(mut self, verify: bool) -> Result<Self, Error<C>> {
        self.easy.doh_ssl_verify_peer(verify).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Tells curl to verify the DoH (DNS-over-HTTPS) server's certificate name
    /// fields against the host name.
    ///
    /// This option is the DoH equivalent of [`Easy2::ssl_verify_host`] and only
    /// affects requests to the DoH server.
    ///
    /// When `doh_ssl_verify_host` is `true`, the SSL certificate provided by
    /// the DoH server must indicate that the server name is the same as the
    /// server name to which you meant to connect to, or the connection fails.
    ///
    /// Curl considers the DoH server the intended one when the Common Name
    /// field or a Subject Alternate Name field in the certificate matches the
    /// host name in the DoH URL to which you told Curl to connect.
    ///
    /// When the verify value is set to `false`, the connection succeeds
    /// regardless of the names used in the certificate. Use that ability with
    /// caution!
    ///
    /// See also [`Easy2::doh_ssl_verify_peer`] to verify the digital signature
    /// of the DoH server certificate. If libcurl is built against NSS and
    /// [`Easy2::doh_ssl_verify_peer`] is `false`, `doh_ssl_verify_host` is also
    /// set to `false` and cannot be overridden.
    ///
    /// By default this option is set to `true` and corresponds to
    /// `CURLOPT_DOH_SSL_VERIFYHOST`.
    pub fn doh_ssl_verify_host(mut self, verify: bool) -> Result<Self, Error<C>> {
        self.easy.doh_ssl_verify_host(verify).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Pass a long as parameter set to 1 to enable or 0 to disable.
    ///
    /// This option determines whether libcurl verifies the status of the DoH
    /// (DNS-over-HTTPS) server cert using the "Certificate Status Request" TLS
    /// extension (aka. OCSP stapling).
    ///
    /// This option is the DoH equivalent of CURLOPT_SSL_VERIFYSTATUS and only
    /// affects requests to the DoH server.
    ///
    /// Note that if this option is enabled but the server does not support the
    /// TLS extension, the verification will fail.
    ///
    /// By default this option is set to `false` and corresponds to
    /// `CURLOPT_DOH_SSL_VERIFYSTATUS`.
    pub fn doh_ssl_verify_status(mut self, verify: bool) -> Result<Self, Error<C>> {
        self.easy.doh_ssl_verify_status(verify).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify the preferred receive buffer size, in bytes.
    ///
    /// This is treated as a request, not an order, and the main point of this
    /// is that the write callback may get called more often with smaller
    /// chunks.
    ///
    /// By default this option is the maximum write size and corresopnds to
    /// `CURLOPT_BUFFERSIZE`.
    pub fn buffer_size(mut self, size: usize) -> Result<Self, Error<C>> {
        self.easy.buffer_size(size).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify the preferred send buffer size, in bytes.
    ///
    /// This is treated as a request, not an order, and the main point of this
    /// is that the read callback may get called more often with smaller
    /// chunks.
    ///
    /// The upload buffer size is by default 64 kilobytes.
    pub fn upload_buffer_size(mut self, size: usize) -> Result<Self, Error<C>> {
        self.easy.upload_buffer_size(size).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // /// Enable or disable TCP Fast Open
    // ///
    // /// By default this options defaults to `false` and corresponds to
    // /// `CURLOPT_TCP_FASTOPEN`
    // pub fn fast_open(mut self, enable: bool) -> Result<Self, Error<C>> {
    // }

    /// Configures whether the TCP_NODELAY option is set, or Nagle's algorithm
    /// is disabled.
    ///
    /// The purpose of Nagle's algorithm is to minimize the number of small
    /// packet's on the network, and disabling this may be less efficient in
    /// some situations.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_TCP_NODELAY`.
    pub fn tcp_nodelay(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.tcp_nodelay(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configures whether TCP keepalive probes will be sent.
    ///
    /// The delay and frequency of these probes is controlled by `tcp_keepidle`
    /// and `tcp_keepintvl`.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_TCP_KEEPALIVE`.
    pub fn tcp_keepalive(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.tcp_keepalive(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configures the TCP keepalive idle time wait.
    ///
    /// This is the delay, after which the connection is idle, keepalive probes
    /// will be sent. Not all operating systems support this.
    ///
    /// By default this corresponds to `CURLOPT_TCP_KEEPIDLE`.
    pub fn tcp_keepidle(mut self, amt: Duration) -> Result<Self, Error<C>> {
        self.easy.tcp_keepidle(amt).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configures the delay between keepalive probes.
    ///
    /// By default this corresponds to `CURLOPT_TCP_KEEPINTVL`.
    pub fn tcp_keepintvl(mut self, amt: Duration) -> Result<Self, Error<C>> {
        self.easy.tcp_keepintvl(amt).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configures the scope for local IPv6 addresses.
    ///
    /// Sets the scope_id value to use when connecting to IPv6 or link-local
    /// addresses.
    ///
    /// By default this value is 0 and corresponds to `CURLOPT_ADDRESS_SCOPE`
    pub fn address_scope(mut self, scope: u32) -> Result<Self, Error<C>> {
        self.easy.address_scope(scope).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // =========================================================================
    // Names and passwords

    /// Configures the username to pass as authentication for this connection.
    ///
    /// By default this value is not set and corresponds to `CURLOPT_USERNAME`.
    pub fn username(mut self, user: &str) -> Result<Self, Error<C>> {
        self.easy.username(user).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configures the password to pass as authentication for this connection.
    ///
    /// By default this value is not set and corresponds to `CURLOPT_PASSWORD`.
    pub fn password(mut self, pass: &str) -> Result<Self, Error<C>> {
        self.easy.password(pass).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set HTTP server authentication methods to try
    ///
    /// If more than one method is set, libcurl will first query the site to see
    /// which authentication methods it supports and then pick the best one you
    /// allow it to use. For some methods, this will induce an extra network
    /// round-trip. Set the actual name and password with the `password` and
    /// `username` methods.
    ///
    /// For authentication with a proxy, see `proxy_auth`.
    ///
    /// By default this value is basic and corresponds to `CURLOPT_HTTPAUTH`.
    pub fn http_auth(mut self, auth: &Auth) -> Result<Self, Error<C>> {
        self.easy.http_auth(auth).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Provides AWS V4 signature authentication on HTTP(S) header.
    ///
    /// `param` is used to create outgoing authentication headers.
    /// Its format is `provider1[:provider2[:region[:service]]]`.
    /// `provider1,\ provider2"` are used for generating auth parameters
    /// such as "Algorithm", "date", "request type" and "signed headers".
    /// `region` is the geographic area of a resources collection. It is
    /// extracted from the host name specified in the URL if omitted.
    /// `service` is a function provided by a cloud. It is extracted
    /// from the host name specified in the URL if omitted.
    ///
    /// Example with "Test:Try", when curl will do the algorithm, it will
    /// generate "TEST-HMAC-SHA256" for "Algorithm", "x-try-date" and
    /// "X-Try-Date" for "date", "test4_request" for "request type", and
    /// "SignedHeaders=content-type;host;x-try-date" for "signed headers".
    /// If you use just "test", instead of "test:try", test will be use
    /// for every strings generated.
    ///
    /// This is a special auth type that can't be combined with the others.
    /// It will override the other auth types you might have set.
    ///
    /// By default this is not set and corresponds to `CURLOPT_AWS_SIGV4`.
    pub fn aws_sigv4(mut self, param: &str) -> Result<Self, Error<C>> {
        self.easy.aws_sigv4(param).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configures the proxy username to pass as authentication for this
    /// connection.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXYUSERNAME`.
    pub fn proxy_username(mut self, user: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_username(user).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configures the proxy password to pass as authentication for this
    /// connection.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXYPASSWORD`.
    pub fn proxy_password(mut self, pass: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_password(pass).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set HTTP proxy authentication methods to try
    ///
    /// If more than one method is set, libcurl will first query the site to see
    /// which authentication methods it supports and then pick the best one you
    /// allow it to use. For some methods, this will induce an extra network
    /// round-trip. Set the actual name and password with the `proxy_password`
    /// and `proxy_username` methods.
    ///
    /// By default this value is basic and corresponds to `CURLOPT_PROXYAUTH`.
    pub fn proxy_auth(mut self, auth: &Auth) -> Result<Self, Error<C>> {
        self.easy.proxy_auth(auth).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Enable .netrc parsing
    ///
    /// By default the .netrc file is ignored and corresponds to `CURL_NETRC_IGNORED`.
    pub fn netrc(mut self, netrc: NetRc) -> Result<Self, Error<C>> {
        self.easy.netrc(netrc).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // =========================================================================
    // HTTP Options

    /// Indicates whether the referer header is automatically updated
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_AUTOREFERER`.
    pub fn autoreferer(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.autoreferer(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Enables automatic decompression of HTTP downloads.
    ///
    /// Sets the contents of the Accept-Encoding header sent in an HTTP request.
    /// This enables decoding of a response with Content-Encoding.
    ///
    /// Currently supported encoding are `identity`, `zlib`, and `gzip`. A
    /// zero-length string passed in will send all accepted encodings.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_ACCEPT_ENCODING`.
    pub fn accept_encoding(mut self, encoding: &str) -> Result<Self, Error<C>> {
        self.easy.accept_encoding(encoding).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Request the HTTP Transfer Encoding.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_TRANSFER_ENCODING`.
    pub fn transfer_encoding(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.transfer_encoding(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Follow HTTP 3xx redirects.
    ///
    /// Indicates whether any `Location` headers in the response should get
    /// followed.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_FOLLOWLOCATION`.
    pub fn follow_location(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.follow_location(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Send credentials to hosts other than the first as well.
    ///
    /// Sends username/password credentials even when the host changes as part
    /// of a redirect.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_UNRESTRICTED_AUTH`.
    pub fn unrestricted_auth(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.unrestricted_auth(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the maximum number of redirects allowed.
    ///
    /// A value of 0 will refuse any redirect.
    ///
    /// By default this option is `-1` (unlimited) and corresponds to
    /// `CURLOPT_MAXREDIRS`.
    pub fn max_redirections(mut self, max: u32) -> Result<Self, Error<C>> {
        self.easy.max_redirections(max).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Make an HTTP PUT request.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_PUT`.
    pub fn put(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.put(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Make an HTTP POST request.
    ///
    /// This will also make the library use the
    /// `Content-Type: application/x-www-form-urlencoded` header.
    ///
    /// POST data can be specified through `post_fields` or by specifying a read
    /// function.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_POST`.
    pub fn post(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.post(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configures the data that will be uploaded as part of a POST.
    ///
    /// Note that the data is copied into this handle and if that's not desired
    /// then the read callbacks can be used instead.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_COPYPOSTFIELDS`.
    pub fn post_fields_copy(mut self, data: &[u8]) -> Result<Self, Error<C>> {
        self.easy.post_fields_copy(data).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configures the size of data that's going to be uploaded as part of a
    /// POST operation.
    ///
    /// This is called automatically as part of `post_fields` and should only
    /// be called if data is being provided in a read callback (and even then
    /// it's optional).
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_POSTFIELDSIZE_LARGE`.
    pub fn post_field_size(mut self, size: u64) -> Result<Self, Error<C>> {
        self.easy.post_field_size(size).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Tells libcurl you want a multipart/formdata HTTP POST to be made and you
    /// instruct what data to pass on to the server in the `form` argument.
    ///
    /// By default this option is set to null and corresponds to
    /// `CURLOPT_HTTPPOST`.
    pub fn httppost(mut self, form: Form) -> Result<Self, Error<C>> {
        self.easy.httppost(form).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Sets the HTTP referer header
    ///
    /// By default this option is not set and corresponds to `CURLOPT_REFERER`.
    pub fn referer(mut self, referer: &str) -> Result<Self, Error<C>> {
        self.easy.referer(referer).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Sets the HTTP user-agent header
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_USERAGENT`.
    pub fn useragent(mut self, useragent: &str) -> Result<Self, Error<C>> {
        self.easy.useragent(useragent).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Add some headers to this HTTP request.
    ///
    /// If you add a header that is otherwise used internally, the value here
    /// takes precedence. If a header is added with no content (like `Accept:`)
    /// the internally the header will get disabled. To add a header with no
    /// content, use the form `MyHeader;` (not the trailing semicolon).
    ///
    /// Headers must not be CRLF terminated. Many replaced headers have common
    /// shortcuts which should be prefered.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_HTTPHEADER`
    pub fn http_headers(mut self, list: List) -> Result<Self, Error<C>> {
        self.easy.http_headers(list).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // /// Add some headers to send to the HTTP proxy.
    // ///
    // /// This function is essentially the same as `http_headers`.
    // ///
    // /// By default this option is not set and corresponds to
    // /// `CURLOPT_PROXYHEADER`
    // pub fn proxy_headers(mut self, list: &'a List) -> Result<Self, Error<C>> {
    //     self.setopt_ptr(curl_sys::CURLOPT_PROXYHEADER, list.raw as *const _)
    // }

    /// Set the contents of the HTTP Cookie header.
    ///
    /// Pass a string of the form `name=contents` for one cookie value or
    /// `name1=val1; name2=val2` for multiple values.
    ///
    /// Using this option multiple times will only make the latest string
    /// override the previous ones. This option will not enable the cookie
    /// engine, use `cookie_file` or `cookie_jar` to do that.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_COOKIE`.
    pub fn cookie(mut self, cookie: &str) -> Result<Self, Error<C>> {
        self.easy.cookie(cookie).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the file name to read cookies from.
    ///
    /// The cookie data can be in either the old Netscape / Mozilla cookie data
    /// format or just regular HTTP headers (Set-Cookie style) dumped to a file.
    ///
    /// This also enables the cookie engine, making libcurl parse and send
    /// cookies on subsequent requests with this handle.
    ///
    /// Given an empty or non-existing file or by passing the empty string ("")
    /// to this option, you can enable the cookie engine without reading any
    /// initial cookies.
    ///
    /// If you use this option multiple times, you just add more files to read.
    /// Subsequent files will add more cookies.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_COOKIEFILE`.
    pub fn cookie_file<P: AsRef<Path>>(mut self, file: P) -> Result<Self, Error<C>> {
        self.easy.cookie_file(file).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the file name to store cookies to.
    ///
    /// This will make libcurl write all internally known cookies to the file
    /// when this handle is dropped. If no cookies are known, no file will be
    /// created. Specify "-" as filename to instead have the cookies written to
    /// stdout. Using this option also enables cookies for this session, so if
    /// you for example follow a location it will make matching cookies get sent
    /// accordingly.
    ///
    /// Note that libcurl doesn't read any cookies from the cookie jar. If you
    /// want to read cookies from a file, use `cookie_file`.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_COOKIEJAR`.
    pub fn cookie_jar<P: AsRef<Path>>(mut self, file: P) -> Result<Self, Error<C>> {
        self.easy.cookie_jar(file).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Start a new cookie session
    ///
    /// Marks this as a new cookie "session". It will force libcurl to ignore
    /// all cookies it is about to load that are "session cookies" from the
    /// previous session. By default, libcurl always stores and loads all
    /// cookies, independent if they are session cookies or not. Session cookies
    /// are cookies without expiry date and they are meant to be alive and
    /// existing for this "session" only.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_COOKIESESSION`.
    pub fn cookie_session(mut self, session: bool) -> Result<Self, Error<C>> {
        self.easy.cookie_session(session).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Add to or manipulate cookies held in memory.
    ///
    /// Such a cookie can be either a single line in Netscape / Mozilla format
    /// or just regular HTTP-style header (Set-Cookie: ...) format. This will
    /// also enable the cookie engine. This adds that single cookie to the
    /// internal cookie store.
    ///
    /// Exercise caution if you are using this option and multiple transfers may
    /// occur. If you use the Set-Cookie format and don't specify a domain then
    /// the cookie is sent for any domain (even after redirects are followed)
    /// and cannot be modified by a server-set cookie. If a server sets a cookie
    /// of the same name (or maybe you've imported one) then both will be sent
    /// on a future transfer to that server, likely not what you intended.
    /// address these issues set a domain in Set-Cookie or use the Netscape
    /// format.
    ///
    /// Additionally, there are commands available that perform actions if you
    /// pass in these exact strings:
    ///
    /// * "ALL" - erases all cookies held in memory
    /// * "SESS" - erases all session cookies held in memory
    /// * "FLUSH" - write all known cookies to the specified cookie jar
    /// * "RELOAD" - reread all cookies from the cookie file
    ///
    /// By default this options corresponds to `CURLOPT_COOKIELIST`
    pub fn cookie_list(mut self, cookie: &str) -> Result<Self, Error<C>> {
        self.easy.cookie_list(cookie).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Ask for a HTTP GET request.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_HTTPGET`.
    pub fn get(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.get(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // /// Ask for a HTTP GET request.
    // ///
    // /// By default this option is `false` and corresponds to `CURLOPT_HTTPGET`.
    // pub fn http_version(mut self, vers: &str) -> Result<Self, Error<C>> {
    //     self.setopt_long(curl_sys::CURLOPT_HTTPGET, enable as c_long)
    // }

    /// Ignore the content-length header.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_IGNORE_CONTENT_LENGTH`.
    pub fn ignore_content_length(mut self, ignore: bool) -> Result<Self, Error<C>> {
        self.easy.ignore_content_length(ignore).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Enable or disable HTTP content decoding.
    ///
    /// By default this option is `true` and corresponds to
    /// `CURLOPT_HTTP_CONTENT_DECODING`.
    pub fn http_content_decoding(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.http_content_decoding(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Enable or disable HTTP transfer decoding.
    ///
    /// By default this option is `true` and corresponds to
    /// `CURLOPT_HTTP_TRANSFER_DECODING`.
    pub fn http_transfer_decoding(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.http_transfer_decoding(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // /// Timeout for the Expect: 100-continue response
    // ///
    // /// By default this option is 1s and corresponds to
    // /// `CURLOPT_EXPECT_100_TIMEOUT_MS`.
    // pub fn expect_100_timeout(mut self, enable: bool) -> Result<Self, Error<C>> {
    //     self.setopt_long(curl_sys::CURLOPT_HTTP_TRANSFER_DECODING,
    //                      enable as c_long)
    // }

    // /// Wait for pipelining/multiplexing.
    // ///
    // /// Tells libcurl to prefer to wait for a connection to confirm or deny that
    // /// it can do pipelining or multiplexing before continuing.
    // ///
    // /// When about to perform a new transfer that allows pipelining or
    // /// multiplexing, libcurl will check for existing connections to re-use and
    // /// pipeline on. If no such connection exists it will immediately continue
    // /// and create a fresh new connection to use.
    // ///
    // /// By setting this option to `true` - having `pipeline` enabled for the
    // /// multi handle this transfer is associated with - libcurl will instead
    // /// wait for the connection to reveal if it is possible to
    // /// pipeline/multiplex on before it continues. This enables libcurl to much
    // /// better keep the number of connections to a minimum when using pipelining
    // /// or multiplexing protocols.
    // ///
    // /// The effect thus becomes that with this option set, libcurl prefers to
    // /// wait and re-use an existing connection for pipelining rather than the
    // /// opposite: prefer to open a new connection rather than waiting.
    // ///
    // /// The waiting time is as long as it takes for the connection to get up and
    // /// for libcurl to get the necessary response back that informs it about its
    // /// protocol and support level.
    // pub fn http_pipewait(mut self, enable: bool) -> Result<Self, Error<C>> {
    // }

    // =========================================================================
    // Protocol Options

    /// Indicates the range that this request should retrieve.
    ///
    /// The string provided should be of the form `N-M` where either `N` or `M`
    /// can be left out. For HTTP transfers multiple ranges separated by commas
    /// are also accepted.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_RANGE`.
    pub fn range(mut self, range: &str) -> Result<Self, Error<C>> {
        self.easy.range(range).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set a point to resume transfer from
    ///
    /// Specify the offset in bytes you want the transfer to start from.
    ///
    /// By default this option is 0 and corresponds to
    /// `CURLOPT_RESUME_FROM_LARGE`.
    pub fn resume_from(mut self, from: u64) -> Result<Self, Error<C>> {
        self.easy.resume_from(from).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set a custom request string
    ///
    /// Specifies that a custom request will be made (e.g. a custom HTTP
    /// method). This does not change how libcurl performs internally, just
    /// changes the string sent to the server.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_CUSTOMREQUEST`.
    pub fn custom_request(mut self, request: &str) -> Result<Self, Error<C>> {
        self.easy.custom_request(request).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Get the modification time of the remote resource
    ///
    /// If true, libcurl will attempt to get the modification time of the
    /// remote document in this operation. This requires that the remote server
    /// sends the time or replies to a time querying command. The `filetime`
    /// function can be used after a transfer to extract the received time (if
    /// any).
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_FILETIME`
    pub fn fetch_filetime(mut self, fetch: bool) -> Result<Self, Error<C>> {
        self.easy.fetch_filetime(fetch).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Indicate whether to download the request without getting the body
    ///
    /// This is useful, for example, for doing a HEAD request.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_NOBODY`.
    pub fn nobody(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.nobody(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the size of the input file to send off.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_INFILESIZE_LARGE`.
    pub fn in_filesize(mut self, size: u64) -> Result<Self, Error<C>> {
        self.easy.in_filesize(size).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Enable or disable data upload.
    ///
    /// This means that a PUT request will be made for HTTP and probably wants
    /// to be combined with the read callback as well as the `in_filesize`
    /// method.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_UPLOAD`.
    pub fn upload(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.upload(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configure the maximum file size to download.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_MAXFILESIZE_LARGE`.
    pub fn max_filesize(mut self, size: u64) -> Result<Self, Error<C>> {
        self.easy.max_filesize(size).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Selects a condition for a time request.
    ///
    /// This value indicates how the `time_value` option is interpreted.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_TIMECONDITION`.
    pub fn time_condition(mut self, cond: TimeCondition) -> Result<Self, Error<C>> {
        self.easy.time_condition(cond).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Sets the time value for a conditional request.
    ///
    /// The value here should be the number of seconds elapsed since January 1,
    /// 1970. To pass how to interpret this value, use `time_condition`.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_TIMEVALUE`.
    pub fn time_value(mut self, val: i64) -> Result<Self, Error<C>> {
        self.easy.time_value(val).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // =========================================================================
    // Connection Options

    /// Set maximum time the request is allowed to take.
    ///
    /// Normally, name lookups can take a considerable time and limiting
    /// operations to less than a few minutes risk aborting perfectly normal
    /// operations.
    ///
    /// If libcurl is built to use the standard system name resolver, that
    /// portion of the transfer will still use full-second resolution for
    /// timeouts with a minimum timeout allowed of one second.
    ///
    /// In unix-like systems, this might cause signals to be used unless
    /// `nosignal` is set.
    ///
    /// Since this puts a hard limit for how long a request is allowed to
    /// take, it has limited use in dynamic use cases with varying transfer
    /// times. You are then advised to explore `low_speed_limit`,
    /// `low_speed_time` or using `progress_function` to implement your own
    /// timeout logic.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_TIMEOUT_MS`.
    pub fn timeout(mut self, timeout: Duration) -> Result<Self, Error<C>> {
        self.easy.timeout(timeout).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the low speed limit in bytes per second.
    ///
    /// This specifies the average transfer speed in bytes per second that the
    /// transfer should be below during `low_speed_time` for libcurl to consider
    /// it to be too slow and abort.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_LOW_SPEED_LIMIT`.
    pub fn low_speed_limit(mut self, limit: u32) -> Result<Self, Error<C>> {
        self.easy.low_speed_limit(limit).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the low speed time period.
    ///
    /// Specifies the window of time for which if the transfer rate is below
    /// `low_speed_limit` the request will be aborted.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_LOW_SPEED_TIME`.
    pub fn low_speed_time(mut self, dur: Duration) -> Result<Self, Error<C>> {
        self.easy.low_speed_time(dur).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Rate limit data upload speed
    ///
    /// If an upload exceeds this speed (counted in bytes per second) on
    /// cumulative average during the transfer, the transfer will pause to keep
    /// the average rate less than or equal to the parameter value.
    ///
    /// By default this option is not set (unlimited speed) and corresponds to
    /// `CURLOPT_MAX_SEND_SPEED_LARGE`.
    pub fn max_send_speed(mut self, speed: u64) -> Result<Self, Error<C>> {
        self.easy.max_send_speed(speed).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Rate limit data download speed
    ///
    /// If a download exceeds this speed (counted in bytes per second) on
    /// cumulative average during the transfer, the transfer will pause to keep
    /// the average rate less than or equal to the parameter value.
    ///
    /// By default this option is not set (unlimited speed) and corresponds to
    /// `CURLOPT_MAX_RECV_SPEED_LARGE`.
    pub fn max_recv_speed(mut self, speed: u64) -> Result<Self, Error<C>> {
        self.easy.max_recv_speed(speed).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the maximum connection cache size.
    ///
    /// The set amount will be the maximum number of simultaneously open
    /// persistent connections that libcurl may cache in the pool associated
    /// with this handle. The default is 5, and there isn't much point in
    /// changing this value unless you are perfectly aware of how this works and
    /// changes libcurl's behaviour. This concerns connections using any of the
    /// protocols that support persistent connections.
    ///
    /// When reaching the maximum limit, curl closes the oldest one in the cache
    /// to prevent increasing the number of open connections.
    ///
    /// By default this option is set to 5 and corresponds to
    /// `CURLOPT_MAXCONNECTS`
    pub fn max_connects(mut self, max: u32) -> Result<Self, Error<C>> {
        self.easy.max_connects(max).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the maximum idle time allowed for a connection.
    ///
    /// This configuration sets the maximum time that a connection inside of the connection cache
    /// can be reused. Any connection older than this value will be considered stale and will
    /// be closed.
    ///
    /// By default, a value of 118 seconds is used.
    pub fn maxage_conn(mut self, max_age: Duration) -> Result<Self, Error<C>> {
        self.easy.maxage_conn(max_age).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Force a new connection to be used.
    ///
    /// Makes the next transfer use a new (fresh) connection by force instead of
    /// trying to re-use an existing one. This option should be used with
    /// caution and only if you understand what it does as it may seriously
    /// impact performance.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_FRESH_CONNECT`.
    pub fn fresh_connect(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.fresh_connect(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Make connection get closed at once after use.
    ///
    /// Makes libcurl explicitly close the connection when done with the
    /// transfer. Normally, libcurl keeps all connections alive when done with
    /// one transfer in case a succeeding one follows that can re-use them.
    /// This option should be used with caution and only if you understand what
    /// it does as it can seriously impact performance.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_FORBID_REUSE`.
    pub fn forbid_reuse(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.forbid_reuse(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Timeout for the connect phase
    ///
    /// This is the maximum time that you allow the connection phase to the
    /// server to take. This only limits the connection phase, it has no impact
    /// once it has connected.
    ///
    /// By default this value is 300 seconds and corresponds to
    /// `CURLOPT_CONNECTTIMEOUT_MS`.
    pub fn connect_timeout(mut self, timeout: Duration) -> Result<Self, Error<C>> {
        self.easy.connect_timeout(timeout).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify which IP protocol version to use
    ///
    /// Allows an application to select what kind of IP addresses to use when
    /// resolving host names. This is only interesting when using host names
    /// that resolve addresses using more than one version of IP.
    ///
    /// By default this value is "any" and corresponds to `CURLOPT_IPRESOLVE`.
    pub fn ip_resolve(mut self, resolve: IpResolve) -> Result<Self, Error<C>> {
        self.easy.ip_resolve(resolve).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify custom host name to IP address resolves.
    ///
    /// Allows specifying hostname to IP mappins to use before trying the
    /// system resolver.
    pub fn resolve(mut self, list: List) -> Result<Self, Error<C>> {
        self.easy.resolve(list).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Configure whether to stop when connected to target server
    ///
    /// When enabled it tells the library to perform all the required proxy
    /// authentication and connection setup, but no data transfer, and then
    /// return.
    ///
    /// The option can be used to simply test a connection to a server.
    ///
    /// By default this value is `false` and corresponds to
    /// `CURLOPT_CONNECT_ONLY`.
    pub fn connect_only(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.connect_only(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // =========================================================================
    // SSL/Security Options

    /// Sets the SSL client certificate.
    ///
    /// The string should be the file name of your client certificate. The
    /// default format is "P12" on Secure Transport and "PEM" on other engines,
    /// and can be changed with `ssl_cert_type`.
    ///
    /// With NSS or Secure Transport, this can also be the nickname of the
    /// certificate you wish to authenticate with as it is named in the security
    /// database. If you want to use a file from the current directory, please
    /// precede it with "./" prefix, in order to avoid confusion with a
    /// nickname.
    ///
    /// When using a client certificate, you most likely also need to provide a
    /// private key with `ssl_key`.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_SSLCERT`.
    pub fn ssl_cert<P: AsRef<Path>>(mut self, cert: P) -> Result<Self, Error<C>> {
        self.easy.ssl_cert(cert).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the SSL client certificate using an in-memory blob.
    ///
    /// The specified byte buffer should contain the binary content of your
    /// client certificate, which will be copied into the handle. The format of
    /// the certificate can be specified with `ssl_cert_type`.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSLCERT_BLOB`.
    pub fn ssl_cert_blob(mut self, blob: &[u8]) -> Result<Self, Error<C>> {
        self.easy.ssl_cert_blob(blob).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify type of the client SSL certificate.
    ///
    /// The string should be the format of your certificate. Supported formats
    /// are "PEM" and "DER", except with Secure Transport. OpenSSL (versions
    /// 0.9.3 and later) and Secure Transport (on iOS 5 or later, or OS X 10.7
    /// or later) also support "P12" for PKCS#12-encoded files.
    ///
    /// By default this option is "PEM" and corresponds to
    /// `CURLOPT_SSLCERTTYPE`.
    pub fn ssl_cert_type(mut self, kind: &str) -> Result<Self, Error<C>> {
        self.easy.ssl_cert_type(kind).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify private keyfile for TLS and SSL client cert.
    ///
    /// The string should be the file name of your private key. The default
    /// format is "PEM" and can be changed with `ssl_key_type`.
    ///
    /// (iOS and Mac OS X only) This option is ignored if curl was built against
    /// Secure Transport. Secure Transport expects the private key to be already
    /// present in the keychain or PKCS#12 file containing the certificate.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_SSLKEY`.
    pub fn ssl_key<P: AsRef<Path>>(mut self, key: P) -> Result<Self, Error<C>> {
        self.easy.ssl_key(key).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify an SSL private key using an in-memory blob.
    ///
    /// The specified byte buffer should contain the binary content of your
    /// private key, which will be copied into the handle. The format of
    /// the private key can be specified with `ssl_key_type`.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSLKEY_BLOB`.
    pub fn ssl_key_blob(mut self, blob: &[u8]) -> Result<Self, Error<C>> {
        self.easy.ssl_key_blob(blob).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set type of the private key file.
    ///
    /// The string should be the format of your private key. Supported formats
    /// are "PEM", "DER" and "ENG".
    ///
    /// The format "ENG" enables you to load the private key from a crypto
    /// engine. In this case `ssl_key` is used as an identifier passed to
    /// the engine. You have to set the crypto engine with `ssl_engine`.
    /// "DER" format key file currently does not work because of a bug in
    /// OpenSSL.
    ///
    /// By default this option is "PEM" and corresponds to
    /// `CURLOPT_SSLKEYTYPE`.
    pub fn ssl_key_type(mut self, kind: &str) -> Result<Self, Error<C>> {
        self.easy.ssl_key_type(kind).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set passphrase to private key.
    ///
    /// This will be used as the password required to use the `ssl_key`.
    /// You never needed a pass phrase to load a certificate but you need one to
    /// load your private key.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_KEYPASSWD`.
    pub fn key_password(mut self, password: &str) -> Result<Self, Error<C>> {
        self.easy.key_password(password).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the SSL Certificate Authorities using an in-memory blob.
    ///
    /// The specified byte buffer should contain the binary content of one
    /// or more PEM-encoded CA certificates, which will be copied into
    /// the handle.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_CAINFO_BLOB`.
    pub fn ssl_cainfo_blob(mut self, blob: &[u8]) -> Result<Self, Error<C>> {
        self.easy.ssl_cainfo_blob(blob).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the SSL Certificate Authorities for HTTPS proxies using an in-memory
    /// blob.
    ///
    /// The specified byte buffer should contain the binary content of one
    /// or more PEM-encoded CA certificates, which will be copied into
    /// the handle.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_CAINFO_BLOB`.
    pub fn proxy_ssl_cainfo_blob(mut self, blob: &[u8]) -> Result<Self, Error<C>> {
        self.easy.proxy_ssl_cainfo_blob(blob).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the SSL engine identifier.
    ///
    /// This will be used as the identifier for the crypto engine you want to
    /// use for your private key.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSLENGINE`.
    pub fn ssl_engine(mut self, engine: &str) -> Result<Self, Error<C>> {
        self.easy.ssl_engine(engine).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Make this handle's SSL engine the default.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSLENGINE_DEFAULT`.
    pub fn ssl_engine_default(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.ssl_engine_default(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // /// Enable TLS false start.
    // ///
    // /// This option determines whether libcurl should use false start during the
    // /// TLS handshake. False start is a mode where a TLS client will start
    // /// sending application data before verifying the server's Finished message,
    // /// thus saving a round trip when performing a full handshake.
    // ///
    // /// By default this option is not set and corresponds to
    // /// `CURLOPT_SSL_FALSESTARTE`.
    // pub fn ssl_false_start(mut self, enable: bool) -> Result<Self, Error<C>> {
    //     self.setopt_long(curl_sys::CURLOPT_SSLENGINE_DEFAULT, enable as c_long)
    // }

    /// Set preferred HTTP version.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_HTTP_VERSION`.
    pub fn http_version(mut self, version: HttpVersion) -> Result<Self, Error<C>> {
        self.easy.http_version(version).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set preferred TLS/SSL version.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSLVERSION`.
    pub fn ssl_version(mut self, version: SslVersion) -> Result<Self, Error<C>> {
        self.easy.ssl_version(version).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set preferred TLS/SSL version when connecting to an HTTPS proxy.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_SSLVERSION`.
    pub fn proxy_ssl_version(mut self, version: SslVersion) -> Result<Self, Error<C>> {
        self.easy.proxy_ssl_version(version).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set preferred TLS/SSL version with minimum version and maximum version.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSLVERSION`.
    pub fn ssl_min_max_version(
        mut self,
        min_version: SslVersion,
        max_version: SslVersion,
    ) -> Result<Self, Error<C>> {
        self.easy
            .ssl_min_max_version(min_version, max_version)
            .map_err(|err| {
                log::trace!("{err}");
                Error::Curl(err)
            })?;
        Ok(self)
    }

    /// Set preferred TLS/SSL version with minimum version and maximum version
    /// when connecting to an HTTPS proxy.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_SSLVERSION`.
    pub fn proxy_ssl_min_max_version(
        mut self,
        min_version: SslVersion,
        max_version: SslVersion,
    ) -> Result<Self, Error<C>> {
        self.easy
            .proxy_ssl_min_max_version(min_version, max_version)
            .map_err(|err| {
                log::trace!("{err}");
                Error::Curl(err)
            })?;
        Ok(self)
    }

    /// Verify the certificate's name against host.
    ///
    /// This should be disabled with great caution! It basically disables the
    /// security features of SSL if it is disabled.
    ///
    /// By default this option is set to `true` and corresponds to
    /// `CURLOPT_SSL_VERIFYHOST`.
    pub fn ssl_verify_host(mut self, verify: bool) -> Result<Self, Error<C>> {
        self.easy.ssl_verify_host(verify).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Verify the certificate's name against host for HTTPS proxy.
    ///
    /// This should be disabled with great caution! It basically disables the
    /// security features of SSL if it is disabled.
    ///
    /// By default this option is set to `true` and corresponds to
    /// `CURLOPT_PROXY_SSL_VERIFYHOST`.
    pub fn proxy_ssl_verify_host(mut self, verify: bool) -> Result<Self, Error<C>> {
        self.easy.proxy_ssl_verify_host(verify).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Verify the peer's SSL certificate.
    ///
    /// This should be disabled with great caution! It basically disables the
    /// security features of SSL if it is disabled.
    ///
    /// By default this option is set to `true` and corresponds to
    /// `CURLOPT_SSL_VERIFYPEER`.
    pub fn ssl_verify_peer(mut self, verify: bool) -> Result<Self, Error<C>> {
        self.easy.ssl_verify_peer(verify).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Verify the peer's SSL certificate for HTTPS proxy.
    ///
    /// This should be disabled with great caution! It basically disables the
    /// security features of SSL if it is disabled.
    ///
    /// By default this option is set to `true` and corresponds to
    /// `CURLOPT_PROXY_SSL_VERIFYPEER`.
    pub fn proxy_ssl_verify_peer(mut self, verify: bool) -> Result<Self, Error<C>> {
        self.easy.proxy_ssl_verify_peer(verify).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // /// Verify the certificate's status.
    // ///
    // /// This option determines whether libcurl verifies the status of the server
    // /// cert using the "Certificate Status Request" TLS extension (aka. OCSP
    // /// stapling).
    // ///
    // /// By default this option is set to `false` and corresponds to
    // /// `CURLOPT_SSL_VERIFYSTATUS`.
    // pub fn ssl_verify_status(mut self, verify: bool) -> Result<Self, Error<C>> {
    //     self.setopt_long(curl_sys::CURLOPT_SSL_VERIFYSTATUS, verify as c_long)
    // }

    /// Specify the path to Certificate Authority (CA) bundle
    ///
    /// The file referenced should hold one or more certificates to verify the
    /// peer with.
    ///
    /// This option is by default set to the system path where libcurl's cacert
    /// bundle is assumed to be stored, as established at build time.
    ///
    /// If curl is built against the NSS SSL library, the NSS PEM PKCS#11 module
    /// (libnsspem.so) needs to be available for this option to work properly.
    ///
    /// By default this option is the system defaults, and corresponds to
    /// `CURLOPT_CAINFO`.
    pub fn cainfo<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error<C>> {
        self.easy.cainfo(path).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the issuer SSL certificate filename
    ///
    /// Specifies a file holding a CA certificate in PEM format. If the option
    /// is set, an additional check against the peer certificate is performed to
    /// verify the issuer is indeed the one associated with the certificate
    /// provided by the option. This additional check is useful in multi-level
    /// PKI where one needs to enforce that the peer certificate is from a
    /// specific branch of the tree.
    ///
    /// This option makes sense only when used in combination with the
    /// [`Easy2::ssl_verify_peer`] option. Otherwise, the result of the check is
    /// not considered as failure.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_ISSUERCERT`.
    pub fn issuer_cert<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error<C>> {
        self.easy.issuer_cert(path).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the issuer SSL certificate filename for HTTPS proxies
    ///
    /// Specifies a file holding a CA certificate in PEM format. If the option
    /// is set, an additional check against the peer certificate is performed to
    /// verify the issuer is indeed the one associated with the certificate
    /// provided by the option. This additional check is useful in multi-level
    /// PKI where one needs to enforce that the peer certificate is from a
    /// specific branch of the tree.
    ///
    /// This option makes sense only when used in combination with the
    /// [`Easy2::proxy_ssl_verify_peer`] option. Otherwise, the result of the
    /// check is not considered as failure.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_ISSUERCERT`.
    pub fn proxy_issuer_cert<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error<C>> {
        self.easy.proxy_issuer_cert(path).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the issuer SSL certificate using an in-memory blob.
    ///
    /// The specified byte buffer should contain the binary content of a CA
    /// certificate in the PEM format. The certificate will be copied into the
    /// handle.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_ISSUERCERT_BLOB`.
    pub fn issuer_cert_blob(mut self, blob: &[u8]) -> Result<Self, Error<C>> {
        self.easy.issuer_cert_blob(blob).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set the issuer SSL certificate for HTTPS proxies using an in-memory blob.
    ///
    /// The specified byte buffer should contain the binary content of a CA
    /// certificate in the PEM format. The certificate will be copied into the
    /// handle.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_ISSUERCERT_BLOB`.
    pub fn proxy_issuer_cert_blob(mut self, blob: &[u8]) -> Result<Self, Error<C>> {
        self.easy.proxy_issuer_cert_blob(blob).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify directory holding CA certificates
    ///
    /// Names a directory holding multiple CA certificates to verify the peer
    /// with. If libcurl is built against OpenSSL, the certificate directory
    /// must be prepared using the openssl c_rehash utility. This makes sense
    /// only when used in combination with the `ssl_verify_peer` option.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_CAPATH`.
    pub fn capath<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error<C>> {
        self.easy.capath(path).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify a Certificate Revocation List file
    ///
    /// Names a file with the concatenation of CRL (in PEM format) to use in the
    /// certificate validation that occurs during the SSL exchange.
    ///
    /// When curl is built to use NSS or GnuTLS, there is no way to influence
    /// the use of CRL passed to help in the verification process. When libcurl
    /// is built with OpenSSL support, X509_V_FLAG_CRL_CHECK and
    /// X509_V_FLAG_CRL_CHECK_ALL are both set, requiring CRL check against all
    /// the elements of the certificate chain if a CRL file is passed.
    ///
    /// This option makes sense only when used in combination with the
    /// [`Easy2::ssl_verify_peer`] option.
    ///
    /// A specific error code (`is_ssl_crl_badfile`) is defined with the
    /// option. It is returned when the SSL exchange fails because the CRL file
    /// cannot be loaded. A failure in certificate verification due to a
    /// revocation information found in the CRL does not trigger this specific
    /// error.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_CRLFILE`.
    pub fn crlfile<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error<C>> {
        self.easy.crlfile(path).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify a Certificate Revocation List file to use when connecting to an
    /// HTTPS proxy.
    ///
    /// Names a file with the concatenation of CRL (in PEM format) to use in the
    /// certificate validation that occurs during the SSL exchange.
    ///
    /// When curl is built to use NSS or GnuTLS, there is no way to influence
    /// the use of CRL passed to help in the verification process. When libcurl
    /// is built with OpenSSL support, X509_V_FLAG_CRL_CHECK and
    /// X509_V_FLAG_CRL_CHECK_ALL are both set, requiring CRL check against all
    /// the elements of the certificate chain if a CRL file is passed.
    ///
    /// This option makes sense only when used in combination with the
    /// [`Easy2::proxy_ssl_verify_peer`] option.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_CRLFILE`.
    pub fn proxy_crlfile<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error<C>> {
        self.easy.proxy_crlfile(path).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Request SSL certificate information
    ///
    /// Enable libcurl's certificate chain info gatherer. With this enabled,
    /// libcurl will extract lots of information and data about the certificates
    /// in the certificate chain used in the SSL connection.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_CERTINFO`.
    pub fn certinfo(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.certinfo(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set pinned public key.
    ///
    /// Pass a pointer to a zero terminated string as parameter. The string can
    /// be the file name of your pinned public key. The file format expected is
    /// "PEM" or "DER". The string can also be any number of base64 encoded
    /// sha256 hashes preceded by "sha256//" and separated by ";"
    ///
    /// When negotiating a TLS or SSL connection, the server sends a certificate
    /// indicating its identity. A public key is extracted from this certificate
    /// and if it does not exactly match the public key provided to this option,
    /// curl will abort the connection before sending or receiving any data.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PINNEDPUBLICKEY`.
    pub fn pinned_public_key(mut self, pubkey: &str) -> Result<Self, Error<C>> {
        self.easy.pinned_public_key(pubkey).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify a source for random data
    ///
    /// The file will be used to read from to seed the random engine for SSL and
    /// more.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_RANDOM_FILE`.
    pub fn random_file<P: AsRef<Path>>(mut self, p: P) -> Result<Self, Error<C>> {
        self.easy.random_file(p).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify EGD socket path.
    ///
    /// Indicates the path name to the Entropy Gathering Daemon socket. It will
    /// be used to seed the random engine for SSL.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_EGDSOCKET`.
    pub fn egd_socket<P: AsRef<Path>>(mut self, p: P) -> Result<Self, Error<C>> {
        self.easy.egd_socket(p).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify ciphers to use for TLS.
    ///
    /// Holds the list of ciphers to use for the SSL connection. The list must
    /// be syntactically correct, it consists of one or more cipher strings
    /// separated by colons. Commas or spaces are also acceptable separators
    /// but colons are normally used, !, - and + can be used as operators.
    ///
    /// For OpenSSL and GnuTLS valid examples of cipher lists include 'RC4-SHA',
    /// ´SHA1+DES´, 'TLSv1' and 'DEFAULT'. The default list is normally set when
    /// you compile OpenSSL.
    ///
    /// You'll find more details about cipher lists on this URL:
    ///
    /// <https://www.openssl.org/docs/apps/ciphers.html>
    ///
    /// For NSS, valid examples of cipher lists include 'rsa_rc4_128_md5',
    /// ´rsa_aes_128_sha´, etc. With NSS you don't add/remove ciphers. If one
    /// uses this option then all known ciphers are disabled and only those
    /// passed in are enabled.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSL_CIPHER_LIST`.
    pub fn ssl_cipher_list(mut self, ciphers: &str) -> Result<Self, Error<C>> {
        self.easy.ssl_cipher_list(ciphers).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Specify ciphers to use for TLS for an HTTPS proxy.
    ///
    /// Holds the list of ciphers to use for the SSL connection. The list must
    /// be syntactically correct, it consists of one or more cipher strings
    /// separated by colons. Commas or spaces are also acceptable separators
    /// but colons are normally used, !, - and + can be used as operators.
    ///
    /// For OpenSSL and GnuTLS valid examples of cipher lists include 'RC4-SHA',
    /// ´SHA1+DES´, 'TLSv1' and 'DEFAULT'. The default list is normally set when
    /// you compile OpenSSL.
    ///
    /// You'll find more details about cipher lists on this URL:
    ///
    /// <https://www.openssl.org/docs/apps/ciphers.html>
    ///
    /// For NSS, valid examples of cipher lists include 'rsa_rc4_128_md5',
    /// ´rsa_aes_128_sha´, etc. With NSS you don't add/remove ciphers. If one
    /// uses this option then all known ciphers are disabled and only those
    /// passed in are enabled.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_SSL_CIPHER_LIST`.
    pub fn proxy_ssl_cipher_list(mut self, ciphers: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_ssl_cipher_list(ciphers).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Enable or disable use of the SSL session-ID cache
    ///
    /// By default all transfers are done using the cache enabled. While nothing
    /// ever should get hurt by attempting to reuse SSL session-IDs, there seem
    /// to be or have been broken SSL implementations in the wild that may
    /// require you to disable this in order for you to succeed.
    ///
    /// This corresponds to the `CURLOPT_SSL_SESSIONID_CACHE` option.
    pub fn ssl_sessionid_cache(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.ssl_sessionid_cache(enable).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set SSL behavior options
    ///
    /// Inform libcurl about SSL specific behaviors.
    ///
    /// This corresponds to the `CURLOPT_SSL_OPTIONS` option.
    pub fn ssl_options(mut self, bits: &SslOpt) -> Result<Self, Error<C>> {
        self.easy.ssl_options(bits).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Set SSL behavior options for proxies
    ///
    /// Inform libcurl about SSL specific behaviors.
    ///
    /// This corresponds to the `CURLOPT_PROXY_SSL_OPTIONS` option.
    pub fn proxy_ssl_options(mut self, bits: &SslOpt) -> Result<Self, Error<C>> {
        self.easy.proxy_ssl_options(bits).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    // /// Stores a private pointer-sized piece of data.
    // ///
    // /// This can be retrieved through the `private` function and otherwise
    // /// libcurl does not tamper with this value. This corresponds to
    // /// `CURLOPT_PRIVATE` and defaults to 0.
    // pub fn set_private(mut self, private: usize) -> Result<Self, Error<C>> {
    //     self.setopt_ptr(curl_sys::CURLOPT_PRIVATE, private as *const _)
    // }
    //
    // /// Fetches this handle's private pointer-sized piece of data.
    // ///
    // /// This corresponds to `CURLINFO_PRIVATE` and defaults to 0.
    // pub fn private(&self) -> Result<usize, Error> {
    //     self.getopt_ptr(curl_sys::CURLINFO_PRIVATE).map(|p| p as usize)
    // }

    // =========================================================================
    // getters

    /// Set maximum time to wait for Expect 100 request before sending body.
    ///
    /// `curl` has internal heuristics that trigger the use of a `Expect`
    /// header for large enough request bodies where the client first sends the
    /// request header along with an `Expect: 100-continue` header. The server
    /// is supposed to validate the headers and respond with a `100` response
    /// status code after which `curl` will send the actual request body.
    ///
    /// However, if the server does not respond to the initial request
    /// within `CURLOPT_EXPECT_100_TIMEOUT_MS` then `curl` will send the
    /// request body anyways.
    ///
    /// The best-case scenario is where the request is invalid and the server
    /// replies with a `417 Expectation Failed` without having to wait for or process
    /// the request body at all. However, this behaviour can also lead to higher
    /// total latency since in the best case, an additional server roundtrip is required
    /// and in the worst case, the request is delayed by `CURLOPT_EXPECT_100_TIMEOUT_MS`.
    ///
    /// More info: <https://curl.se/libcurl/c/CURLOPT_EXPECT_100_TIMEOUT_MS.html>
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_EXPECT_100_TIMEOUT_MS`.
    pub fn expect_100_timeout(mut self, timeout: Duration) -> Result<Self, Error<C>> {
        self.easy.expect_100_timeout(timeout).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Wait for pipelining/multiplexing
    ///
    /// Set wait to `true` to tell libcurl to prefer to wait for a connection to
    /// confirm or deny that it can do pipelining or multiplexing before
    /// continuing.
    ///
    /// When about to perform a new transfer that allows pipelining or
    /// multiplexing, libcurl will check for existing connections to re-use and
    /// pipeline on. If no such connection exists it will immediately continue
    /// and create a fresh new connection to use.
    ///
    /// By setting this option to `true` - and having `pipelining(true, true)`
    /// enabled for the multi handle this transfer is associated with - libcurl
    /// will instead wait for the connection to reveal if it is possible to
    /// pipeline/multiplex on before it continues. This enables libcurl to much
    /// better keep the number of connections to a minimum when using pipelining
    /// or multiplexing protocols.
    ///
    /// The effect thus becomes that with this option set, libcurl prefers to
    /// wait and re-use an existing connection for pipelining rather than the
    /// opposite: prefer to open a new connection rather than waiting.
    ///
    /// The waiting time is as long as it takes for the connection to get up and
    /// for libcurl to get the necessary response back that informs it about its
    /// protocol and support level.
    ///
    /// This corresponds to the `CURLOPT_PIPEWAIT` option.
    pub fn pipewait(mut self, wait: bool) -> Result<Self, Error<C>> {
        self.easy.pipewait(wait).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Allow HTTP/0.9 compliant responses
    ///
    /// Set allow to `true` to tell libcurl to allow HTTP/0.9 responses. A HTTP/0.9
    /// response is a server response entirely without headers and only a body.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_HTTP09_ALLOWED`.
    pub fn http_09_allowed(mut self, allow: bool) -> Result<Self, Error<C>> {
        self.easy.http_09_allowed(allow).map_err(|err| {
            log::trace!("{err}");
            Error::Curl(err)
        })?;
        Ok(self)
    }

    /// Finalizes your build to proceed in performing CURL operation.
    pub fn finalize(self) -> HttpClient<C, Perform> {
        HttpClient::<C, Perform> {
            curl: self.curl,
            easy: self.easy,
            _state: Perform,
        }
    }
}

impl<C> HttpClient<C, Perform>
where
    C: Handler + std::fmt::Debug + Send,
{
    /// This will send the request asynchronously,
    /// and return the underlying [`Easy2<C>`](https://docs.rs/curl/latest/curl/easy/struct.Easy2.html) useful if you
    /// want to decide how to transform the response yourself.
    pub async fn perform(self) -> Result<Easy2<C>, Error<C>> {
        self.curl.send_request(self.easy).await
    }
}
