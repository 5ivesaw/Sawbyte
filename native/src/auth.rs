//! Microsoft device authorization and Minecraft session authentication.
use crate::wire::{bad, Result, Reader, Tx, Rx, blob};
use md5::{Digest, Md5};
use rand::RngCore;
use reqwest::blocking::Client;
use rsa::{RsaPublicKey, Pkcs1v15Encrypt, pkcs8::DecodePublicKey};
use serde_json::{json, Value};
use sha1::Sha1;
use std::{path::Path, time::{Duration, Instant}, io::Write};

pub struct Account { pub name: String, pub id: [u8; 16], token: Option<String> }
impl Account {
    pub fn offline(name: &str) -> Result<Self> {
        if name.is_empty() || name.len() > 16 || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') { return Err(bad("invalid offline username")); }
        let mut id: [u8;16] = Md5::digest(format!("OfflinePlayer:{name}")).into();
        id[6] = (id[6] & 15) | 48; id[8] = (id[8] & 63) | 128;
        Ok(Self { name: name.into(), id, token: None })
    }
    pub fn token_file(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        if file.metadata()?.len() > 65536 { return Err(bad("token file too large")); }
        let v: Value = serde_json::from_reader(file)?;
        let id = text(&v,"id")?;
        if id.len()!=32 { return Err(bad("invalid profile id")); }
        let mut bytes=[0;16]; for (i,b) in bytes.iter_mut().enumerate() { *b=u8::from_str_radix(&id[i*2..i*2+2],16)?; }
        Ok(Self { name:text(&v,"name")?.into(), id:bytes, token:Some(text(&v,"access_token")?.into()) })
    }
    pub fn encryption(&self, r: &mut Reader<'_>, rx: &mut Rx, tx: &mut Tx) -> Result<()> {
        let server_id = r.string()?; let public = r.blob()?; let challenge = r.blob()?;
        let authenticate = r.byte()? != 0;
        let mut secret=[0;16]; rand::thread_rng().fill_bytes(&mut secret);
        if authenticate {
            let token=self.token.as_ref().ok_or_else(|| bad("server requires Microsoft login; use login then --account"))?;
            let mut digest=Sha1::new(); digest.update(server_id.as_bytes()); digest.update(secret); digest.update(&public);
            let hash=java_hash(digest.finalize().into());
            let id=self.id.iter().map(|b|format!("{b:02x}")).collect::<String>();
            http()?.post("https://sessionserver.mojang.com/session/minecraft/join")
                .json(&json!({"accessToken":token,"selectedProfile":id,"serverId":hash}))
                .send()?.error_for_status()?;
        }
        let key=RsaPublicKey::from_public_key_der(&public)?;
        let mut reply=Vec::new();
        blob(&key.encrypt(&mut rand::thread_rng(),Pkcs1v15Encrypt,&secret)?,&mut reply);
        blob(&key.encrypt(&mut rand::thread_rng(),Pkcs1v15Encrypt,&challenge)?,&mut reply);
        tx.packet(1,&reply)?; rx.encrypt(&secret); tx.encrypt(&secret); Ok(())
    }
}
fn java_hash(mut bytes: [u8;20]) -> String {
    let negative=bytes[0]&128!=0;
    if negative { let mut carry=1u16; for b in bytes.iter_mut().rev() { let n=(!*b) as u16+carry; *b=n as u8; carry=n>>8; } }
    let hex=bytes.iter().map(|b|format!("{b:02x}")).collect::<String>();
    let digits=hex.trim_start_matches('0'); format!("{}{}",if negative {"-"} else {""},if digits.is_empty() {"0"} else {digits})
}
fn http() -> Result<Client> { Ok(Client::builder().timeout(Duration::from_secs(30)).build()?) }
fn text<'a>(value: &'a Value,key:&str) -> Result<&'a str> { value[key].as_str().ok_or_else(||bad("authentication response missing expected field")) }

/// Public application ID must be registered for Xbox/Minecraft authentication.
/// No borrowed launcher application ID and no password collection.
pub fn login(client_id:&str,path:&Path) -> Result<()> {
    let http=http()?;
    let base="https://login.microsoftonline.com/consumers/oauth2/v2.0";
    let code:Value=http.post(format!("{base}/devicecode")).form(&[("client_id",client_id),("scope","XboxLive.signin offline_access")]).send()?.error_for_status()?.json()?;
    println!("Open {} and enter {}",text(&code,"verification_uri")?,text(&code,"user_code")?);
    let deadline=Instant::now()+Duration::from_secs(code["expires_in"].as_u64().unwrap_or(900).min(1800));
    let mut interval=code["interval"].as_u64().unwrap_or(5).clamp(1,60);
    let msa=loop {
        if Instant::now()>=deadline {return Err(bad("device login expired"));}
        std::thread::sleep(Duration::from_secs(interval));
        let response:Value=http.post(format!("{base}/token")).form(&[("client_id",client_id),("grant_type","urn:ietf:params:oauth:grant-type:device_code"),("device_code",text(&code,"device_code")?)]).send()?.json()?;
        match response["error"].as_str() {
            Some("authorization_pending")=>continue,
            Some("slow_down")=>{interval=(interval+5).min(60);continue;},
            Some(_)=>return Err(bad("Microsoft login declined or application not permitted")),
            None=>break response,
        }
    };
    let xbox:Value=http.post("https://user.auth.xboxlive.com/user/authenticate").json(&json!({
        "Properties":{"AuthMethod":"RPS","SiteName":"user.auth.xboxlive.com","RpsTicket":format!("d={}",text(&msa,"access_token")?)},
        "RelyingParty":"http://auth.xboxlive.com","TokenType":"JWT"
    })).send()?.error_for_status()?.json()?;
    let xsts:Value=http.post("https://xsts.auth.xboxlive.com/xsts/authorize").json(&json!({
        "Properties":{"SandboxId":"RETAIL","UserTokens":[text(&xbox,"Token")?]},
        "RelyingParty":"rp://api.minecraftservices.com/","TokenType":"JWT"
    })).send()?.error_for_status()?.json()?;
    let uhs=text(&xsts["DisplayClaims"]["xui"][0],"uhs")?;
    let mc:Value=http.post("https://api.minecraftservices.com/authentication/login_with_xbox").json(&json!({"identityToken":format!("XBL3.0 x={uhs};{}",text(&xsts,"Token")?)})).send()?.error_for_status()?.json()?;
    let token=text(&mc,"access_token")?;
    let profile:Value=http.get("https://api.minecraftservices.com/minecraft/profile").bearer_auth(token).send()?.error_for_status()?.json()?;
    let contents=json!({"name":text(&profile,"name")?,"id":text(&profile,"id")?,"access_token":token});
    let mut options=std::fs::OpenOptions::new(); options.write(true).create_new(true);
    #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
    let mut file=options.open(path)?; file.write_all(serde_json::to_string(&contents)?.as_bytes())?; file.sync_all()?;
    println!("Signed in as {}. Saved a short-lived access token; keep this file private.",text(&profile,"name")?); Ok(())
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn signed_hash_vectors() {
        assert_eq!(java_hash(Sha1::digest(b"Notch").into()),"4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48");
        assert_eq!(java_hash(Sha1::digest(b"jeb_").into()),"-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1");
    }
}
