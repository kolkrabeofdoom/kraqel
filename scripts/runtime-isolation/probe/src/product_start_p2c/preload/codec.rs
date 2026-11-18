//! Strict bounded canonical model formats. Not accepted by any native store or start route.
use super::*;
struct Writer(Vec<u8>);
impl Writer {
    fn new(domain: &[u8]) -> Self {
        Self(domain.to_vec())
    }
    fn byte(&mut self, v: u8) {
        self.0.push(v);
    }
    fn bool(&mut self, v: bool) {
        self.byte(u8::from(v));
    }
    fn u16(&mut self, v: u16) {
        self.0.extend(v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend(v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend(v.to_le_bytes());
    }
    fn string(&mut self, v: &str) {
        self.u16(v.len() as u16);
        self.0.extend(v.as_bytes());
    }
    fn set(&mut self, v: &BTreeSet<Mitigation>) {
        self.byte(v.len() as u8);
        for m in v {
            self.byte(*m as u8);
        }
    }
    fn generation(&mut self, v: Generation) {
        self.u32(v.pid);
        self.u64(v.created);
    }
    fn request(&mut self, v: &ModelRequestV1) {
        self.u16(v.context.version);
        self.0.extend(v.context.policy_sha256);
        self.u64(v.context.revision);
        self.generation(v.context.guardian);
        self.0.extend(v.context.job);
        self.0.extend(v.launch);
        self.0.extend(v.nonce);
    }
    fn done(self) -> Result<Vec<u8>> {
        if self.0.len() > MAX_BYTES {
            Err(Error::Limit)
        } else {
            Ok(self.0)
        }
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], domain: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(Error::Limit);
        }
        let mut r = Self { bytes, offset: 0 };
        if r.take(domain.len())? != domain {
            return Err(Error::Shape);
        }
        Ok(r)
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.offset.checked_add(n).ok_or(Error::Limit)?;
        let v = self.bytes.get(self.offset..end).ok_or(Error::Shape)?;
        self.offset = end;
        Ok(v)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.take(N)?.try_into().map_err(|_| Error::Shape)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn bool(&mut self) -> Result<bool> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Shape),
        }
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.array()?))
    }
    fn architecture(&mut self) -> Result<Architecture> {
        match self.byte()? {
            1 => Ok(Architecture::WindowsX64),
            _ => Err(Error::Shape),
        }
    }
    fn purpose(&mut self) -> Result<Purpose> {
        match self.byte()? {
            1 => Ok(Purpose::SyntheticModel),
            _ => Err(Error::Shape),
        }
    }
    fn children(&mut self) -> Result<Children> {
        match self.byte()? {
            1 => Ok(Children::Denied),
            2 => Ok(Children::RequiresBoundRole),
            _ => Err(Error::Shape),
        }
    }
    fn load(&mut self) -> Result<LoadRequirement> {
        match self.byte()? {
            1 => Ok(LoadRequirement::ExactPackageAndPlatform),
            _ => Err(Error::Shape),
        }
    }
    fn role(&mut self) -> Result<Role> {
        match self.byte()? {
            1 => Ok(Role::Root),
            2 => Ok(Role::Browser),
            3 => Ok(Role::Renderer),
            4 => Ok(Role::Gpu),
            5 => Ok(Role::Utility),
            6 => Ok(Role::CrashHandler),
            _ => Err(Error::Shape),
        }
    }
    fn string(&mut self) -> Result<String> {
        let n = usize::from(self.u16()?);
        if n == 0 || n > 1024 {
            return Err(Error::Limit);
        }
        Ok(std::str::from_utf8(self.take(n)?)
            .map_err(|_| Error::Shape)?
            .to_owned())
    }
    fn set(&mut self) -> Result<BTreeSet<Mitigation>> {
        let n = self.byte()?;
        if n > 7 {
            return Err(Error::Limit);
        }
        let mut set = BTreeSet::new();
        for _ in 0..n {
            let m = match self.byte()? {
                1 => Mitigation::NoRemoteImages,
                2 => Mitigation::NoLowLabelImages,
                3 => Mitigation::PreferSystem32,
                4 => Mitigation::MicrosoftSignedOnly,
                5 => Mitigation::StoreSignedOnly,
                6 => Mitigation::ProhibitDynamicCode,
                7 => Mitigation::DisableExtensionPoints,
                _ => return Err(Error::Shape),
            };
            if !set.insert(m) {
                return Err(Error::Shape);
            }
        }
        Ok(set)
    }
    fn generation(&mut self) -> Result<Generation> {
        Ok(Generation {
            pid: self.u32()?,
            created: self.u64()?,
        })
    }
    fn request(&mut self) -> Result<ModelRequestV1> {
        Ok(ModelRequestV1 {
            context: ModelContextV1 {
                version: self.u16()?,
                policy_sha256: self.array()?,
                revision: self.u64()?,
                guardian: self.generation()?,
                job: self.array()?,
            },
            launch: self.array()?,
            nonce: self.array()?,
        })
    }
    fn canonical(&self, encoded: &[u8]) -> Result<()> {
        if self.offset != self.bytes.len() || encoded != self.bytes {
            Err(Error::Shape)
        } else {
            Ok(())
        }
    }
    fn model(bytes: &'a [u8], kind: u8) -> Result<Self> {
        let mut r = Self::new(bytes, MODEL_DOMAIN)?;
        if r.byte()? != kind {
            return Err(Error::Shape);
        }
        Ok(r)
    }
}
impl PreloadPolicyV1 {
    pub(crate) fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut w = Writer::new(POLICY_DOMAIN);
        w.u16(self.version);
        w.byte(self.purpose as u8);
        w.byte(self.architecture as u8);
        w.u32(self.min_os_build);
        for d in [
            self.package_sha256,
            self.controller_sha256,
            self.platform_sha256,
            self.environment_sha256,
        ] {
            w.0.extend(d);
        }
        w.u16(self.max_processes);
        w.byte(self.roles.len() as u8);
        let mut roles: Vec<_> = self.roles.iter().collect();
        roles.sort_by_key(|r| r.role);
        for r in roles {
            w.byte(r.role as u8);
            w.string(&r.image);
            w.0.extend(r.image_sha256);
            w.u16(r.max_instances);
            for d in [r.arguments_sha256, r.token_sha256, r.handles_sha256] {
                w.0.extend(d);
            }
            w.set(&r.required);
            w.byte(r.children as u8);
            w.byte(r.load as u8);
        }
        w.done()
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes, POLICY_DOMAIN)?;
        let (version, purpose, architecture, min_os_build) =
            (r.u16()?, r.purpose()?, r.architecture()?, r.u32()?);
        let (package_sha256, controller_sha256, platform_sha256, environment_sha256) =
            (r.array()?, r.array()?, r.array()?, r.array()?);
        let max_processes = r.u16()?;
        let n = r.byte()?;
        if !(1..=6).contains(&n) {
            return Err(Error::Limit);
        }
        let mut roles = Vec::new();
        for _ in 0..n {
            roles.push(PreloadRoleRuleV1 {
                role: r.role()?,
                image: r.string()?,
                image_sha256: r.array()?,
                max_instances: r.u16()?,
                arguments_sha256: r.array()?,
                token_sha256: r.array()?,
                handles_sha256: r.array()?,
                required: r.set()?,
                children: r.children()?,
                load: r.load()?,
            });
        }
        let p = Self {
            version,
            purpose,
            architecture,
            min_os_build,
            package_sha256,
            controller_sha256,
            platform_sha256,
            environment_sha256,
            max_processes,
            roles,
        };
        r.canonical(&p.encode()?)?;
        Ok(p)
    }
}
impl PreloadSupportV1 {
    pub(crate) fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut w = Writer::new(MODEL_DOMAIN);
        w.byte(1);
        w.u16(self.version);
        w.byte(self.architecture as u8);
        w.u32(self.os_build);
        w.0.extend(self.policy_sha256);
        w.bool(self.query_complete);
        w.bool(self.child_policy_supported);
        w.bool(self.simulated_host_anchor);
        w.set(&self.supported);
        w.done()
    }
    pub(crate) fn digest(&self) -> Result<Digest> {
        Ok(Sha256::digest(self.encode()?).into())
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::model(bytes, 1)?;
        let p = Self {
            version: r.u16()?,
            architecture: r.architecture()?,
            os_build: r.u32()?,
            policy_sha256: r.array()?,
            query_complete: r.bool()?,
            child_policy_supported: r.bool()?,
            simulated_host_anchor: r.bool()?,
            supported: r.set()?,
        };
        r.canonical(&p.encode()?)?;
        Ok(p)
    }
}
impl PreloadWitnessV1 {
    pub(crate) fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut w = Writer::new(MODEL_DOMAIN);
        w.byte(2);
        w.u16(self.version);
        w.request(&self.request);
        w.generation(self.root);
        for d in [
            self.support_sha256,
            self.image_sha256,
            self.arguments_sha256,
            self.token_sha256,
            self.handles_sha256,
        ] {
            w.0.extend(d);
        }
        w.bool(self.readback_complete);
        w.set(&self.enforced);
        w.set(&self.audited);
        w.byte(self.actual_children as u8);
        for b in [
            self.thread_opt_out,
            self.remote_downgrade,
            self.simulated_exact_load,
            self.simulated_role_gate,
        ] {
            w.bool(b);
        }
        w.done()
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::model(bytes, 2)?;
        let p = Self {
            version: r.u16()?,
            request: r.request()?,
            root: r.generation()?,
            support_sha256: r.array()?,
            image_sha256: r.array()?,
            arguments_sha256: r.array()?,
            token_sha256: r.array()?,
            handles_sha256: r.array()?,
            readback_complete: r.bool()?,
            enforced: r.set()?,
            audited: r.set()?,
            actual_children: r.children()?,
            thread_opt_out: r.bool()?,
            remote_downgrade: r.bool()?,
            simulated_exact_load: r.bool()?,
            simulated_role_gate: r.bool()?,
        };
        r.canonical(&p.encode()?)?;
        Ok(p)
    }
}
impl ModelEndV1 {
    pub(crate) fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut w = Writer::new(MODEL_DOMAIN);
        w.byte(3);
        w.u16(self.version);
        w.request(&self.request);
        w.generation(self.root);
        w.bool(self.root_ended);
        w.bool(self.same_job_drained);
        w.done()
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::model(bytes, 3)?;
        let p = Self {
            version: r.u16()?,
            request: r.request()?,
            root: r.generation()?,
            root_ended: r.bool()?,
            same_job_drained: r.bool()?,
        };
        r.canonical(&p.encode()?)?;
        Ok(p)
    }
}
