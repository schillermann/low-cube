use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, UdpSocket};
use std::time::{Duration, Instant};

const UPNP_SERVICES: &[&str] = &[
    "urn:schemas-upnp-org:service:WANIPConnection:1",
    "urn:schemas-upnp-org:service:WANPPPConnection:1",
];

/// Local router NAT-PMP & UPnP IGD gateway lease (supporting FRITZ!Box status & port mapping).
#[derive(Clone, Copy, Debug)]
pub struct GatewayLease {
    local_ip: Ipv4Addr,
    port: u16,
}

impl GatewayLease {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(local_ip: Ipv4Addr, port: u16) -> Self {
        Self { local_ip, port }
    }

    fn likely_gateway(&self) -> Ipv4Addr {
        let oct = self.local_ip.octets();
        Ipv4Addr::new(oct[0], oct[1], oct[2], 1)
    }

    /// Requests a NAT-PMP / UPnP UDP port lease on the home router when permitted.
    pub fn request_port_mapping(&self) {
        let _ = self.try_nat_pmp_lease();
        if let Some((addr, path, service)) = self.discover_upnp_control(Duration::from_millis(180)) {
            let body = format!(
                "<?xml version=\"1.0\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" \
                s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body>\
                <u:AddPortMapping xmlns:u=\"{service}\">\
                <NewRemoteHost></NewRemoteHost><NewExternalPort>{port}</NewExternalPort>\
                <NewProtocol>UDP</NewProtocol><NewInternalPort>{port}</NewInternalPort>\
                <NewInternalClient>{ip}</NewInternalClient><NewEnabled>1</NewEnabled>\
                <NewPortMappingDescription>LowCube</NewPortMappingDescription>\
                <NewLeaseDuration>3600</NewLeaseDuration>\
                </u:AddPortMapping></s:Body></s:Envelope>",
                port = self.port,
                ip = self.local_ip
            );
            let _ = Self::soap_call(addr, &path, service, "AddPortMapping", &body);
        }
    }

    /// Queries the home router's WAN IPv4 locally (works on default FRITZ!Box even with UPnP forwarding off).
    pub fn query_external_ip(&self, timeout: Duration) -> Option<Ipv4Addr> {
        if let Some(ip) = self.try_nat_pmp_external_ip() {
            return Some(ip);
        }
        let (addr, path, service) = self.discover_upnp_control(timeout)?;
        let body = format!(
            "<?xml version=\"1.0\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" \
            s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body>\
            <u:GetExternalIPAddress xmlns:u=\"{service}\"></u:GetExternalIPAddress>\
            </s:Body></s:Envelope>"
        );
        let xml = Self::soap_call(addr, &path, service, "GetExternalIPAddress", &body)?;
        let start = xml.find("<NewExternalIPAddress>")? + "<NewExternalIPAddress>".len();
        let end = xml[start..].find("</NewExternalIPAddress>")? + start;
        let parsed: Ipv4Addr = xml[start..end].trim().parse().ok()?;
        if parsed.is_unspecified() || parsed.is_private() || parsed.is_loopback() {
            None
        } else {
            Some(parsed)
        }
    }

    fn try_nat_pmp_lease(&self) -> Option<u16> {
        let sock = UdpSocket::bind("0.0.0.0:0").ok()?;
        let _ = sock.set_read_timeout(Some(Duration::from_millis(80)));
        let gw = SocketAddr::V4(SocketAddrV4::new(self.likely_gateway(), 5351));
        let p = self.port.to_be_bytes();
        let req = [0, 1, 0, 0, p[0], p[1], p[0], p[1], 0, 0, 0x0E, 0x10];
        sock.send_to(&req, gw).ok()?;
        let mut resp = [0u8; 16];
        let (len, _) = sock.recv_from(&mut resp).ok()?;
        if len >= 16 && resp[1] == 129 && u16::from_be_bytes([resp[2], resp[3]]) == 0 {
            Some(u16::from_be_bytes([resp[10], resp[11]]))
        } else {
            None
        }
    }

    fn try_nat_pmp_external_ip(&self) -> Option<Ipv4Addr> {
        let sock = UdpSocket::bind("0.0.0.0:0").ok()?;
        let _ = sock.set_read_timeout(Some(Duration::from_millis(60)));
        let gw = SocketAddr::V4(SocketAddrV4::new(self.likely_gateway(), 5351));
        sock.send_to(&[0u8, 0u8], gw).ok()?;
        let mut resp = [0u8; 16];
        let (len, _) = sock.recv_from(&mut resp).ok()?;
        if len >= 12 && resp[1] == 128 && u16::from_be_bytes([resp[2], resp[3]]) == 0 {
            let ip = Ipv4Addr::new(resp[8], resp[9], resp[10], resp[11]);
            if !ip.is_unspecified() && !ip.is_private() {
                return Some(ip);
            }
        }
        None
    }

    fn discover_upnp_control(&self, timeout: Duration) -> Option<(SocketAddr, String, &'static str)> {
        let sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(self.local_ip, 0))).ok()?;
        let _ = sock.set_nonblocking(true);
        let ssdp = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(239, 255, 255, 250), 1900));
        for &svc in UPNP_SERVICES {
            let query = format!(
                "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\nMX: 1\r\nST: {svc}\r\n\r\n"
            );
            let _ = sock.send_to(query.as_bytes(), ssdp);
        }
        let deadline = Instant::now() + timeout;
        let mut buf = [0u8; 1500];
        while Instant::now() < deadline {
            if let Ok((len, _)) = sock.recv_from(&mut buf) {
                let text = String::from_utf8_lossy(&buf[..len]);
                if let Some(target) = Self::resolve_ssdp_location(&text) {
                    return Some(target);
                }
            }
            std::thread::sleep(Duration::from_millis(8));
        }
        None
    }

    fn resolve_ssdp_location(headers: &str) -> Option<(SocketAddr, String, &'static str)> {
        let loc_line = headers
            .lines()
            .find(|l| l.len() > 9 && l[..9].eq_ignore_ascii_case("location:"))?;
        let url = loc_line[9..].trim().strip_prefix("http://")?;
        let (host_port, desc_path) = match url.split_once('/') {
            Some((h, p)) => (h, format!("/{p}")),
            None => (url, "/".to_string()),
        };
        let addr: SocketAddr = host_port.parse().ok()?;
        let xml = Self::http_get(addr, &desc_path)?;
        for &svc in UPNP_SERVICES {
            if let Some(svc_pos) = xml.find(svc) {
                let tail = &xml[svc_pos..];
                let ctrl_start = tail.find("<controlURL>")? + "<controlURL>".len();
                let ctrl_end = tail[ctrl_start..].find("</controlURL>")? + ctrl_start;
                let ctrl = tail[ctrl_start..ctrl_end].trim();
                let path = if ctrl.starts_with('/') { ctrl.to_string() } else { format!("/{ctrl}") };
                return Some((addr, path, svc));
            }
        }
        None
    }

    fn http_get(addr: SocketAddr, path: &str) -> Option<String> {
        let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(120)).ok()?;
        let _ = stream.set_read_timeout(Some(Duration::from_millis(150)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(120)));
        let req = format!("GET {path} HTTP/1.0\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
        stream.write_all(req.as_bytes()).ok()?;
        let mut resp = String::new();
        let _ = stream.read_to_string(&mut resp);
        if resp.is_empty() { None } else { Some(resp) }
    }

    fn soap_call(addr: SocketAddr, path: &str, service: &str, action: &str, body: &str) -> Option<String> {
        let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(120)).ok()?;
        let _ = stream.set_read_timeout(Some(Duration::from_millis(150)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(120)));
        let req = format!(
            "POST {path} HTTP/1.0\r\nHost: {addr}\r\nContent-Type: text/xml; charset=\"utf-8\"\r\n\
            SOAPAction: \"{service}#{action}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(req.as_bytes()).ok()?;
        let mut resp = String::new();
        let _ = stream.read_to_string(&mut resp);
        if resp.contains("200 OK") { Some(resp) } else { None }
    }
}
