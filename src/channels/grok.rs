use std::fmt;
use std::fmt::Error;

use grok::{Grok, Pattern};
use serde::export::Formatter;
use serde_json::{Map, Value};

use crate::channels::core::*;
use crate::channels::core::Mutator;

#[derive(Debug)]
pub struct Grok_ {
    pub modifier: Mutation,
    // pub custom_patterns: Option<&'a Vec<Value>>, //TODO
    pub pattern: String,
}

impl Mutator for Grok_ {
    fn mutate(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        let compiled = match self.compile() {
            Ok(p) => p,
            Err(err) => return Some(err),
        };


        let maybe_value = v.get(&self.modifier.field);
        let value = match maybe_value {
            None => return Some(anyhow!("value '{}' not found", self.modifier.field)),
            Some(v) => v,
        };

        let incoming_value = match value {
            Value::String(s) => s,
            _ => return Some(anyhow!("grok uses a string value"))
        };

        //  Match the compiled pattern against a string
        let matches = match compiled.match_against(incoming_value.as_str()) {
            Some(value) => value,
            None => return Some(anyhow!("no matches")),
        };


        let mut values: Vec<(String, Value)> = Vec::new();

        for (x, y) in matches.iter() {
            let k = x.clone();
            let val = y.clone();
            values.push((k.to_string(), Value::from(val)))
        }

        for (x,y) in values.into_iter(){
            v.insert(x, y);
        }

        None
    }
}

impl Grok_ {
    fn compile(&self) -> Result<Pattern, anyhow::Error> {
        let mut grok = Grok::default();

        grok.insert_definition("USERNAME", r#"[a-zA-Z0-9._-]+"#);
        grok.insert_definition("USER", r#"%{USERNAME}"#);
        grok.insert_definition("INT", r#"(?:[+-]?(?:[0-9]+))"#);
        grok.insert_definition("BASE10NUM", r#"(?<![0-9.+-])(?>[+-]?(?:(?:[0-9]+(?:\.[0-9]+)?)|(?:\.[0-9]+)))"#);
        grok.insert_definition("NUMBER", r#"(?:%{BASE10NUM})"#);
        grok.insert_definition("BASE16NUM", r#"(?<![0-9A-Fa-f])(?:[+-]?(?:0x)?(?:[0-9A-Fa-f]+))"#);
        grok.insert_definition("BASE16FLOAT", r#"\b(?<![0-9A-Fa-f.])(?:[+-]?(?:0x)?(?:(?:[0-9A-Fa-f]+(?:\.[0-9A-Fa-f]*)?)|(?:\.[0-9A-Fa-f]+)))\b"#);
        grok.insert_definition("POSINT", r#"\b(?:[1-9][0-9]*)\b"#);
        grok.insert_definition("NONNEGINT", r#"\b(?:[0-9]+)\b"#);
        grok.insert_definition("WORD", r#"\b\w+\b"#);
        grok.insert_definition("NOTSPACE", r#"\S+"#);
        grok.insert_definition("SPACE", r#"\s*"#);
        grok.insert_definition("DATA", r#".*?"#);
        grok.insert_definition("GREEDYDATA", r#".*"#);
        grok.insert_definition("QUOTEDSTRING", r#"(?>(?<!\\)(?>"(?>\\.|[^\\"]+)+"|""|(?>'(?>\\.|[^\\']+)+')|''|(?>`(?>\\.|[^\\`]+)+`)|``))"#);
        grok.insert_definition("UUID", r#"[A-Fa-f0-9]{8}-(?:[A-Fa-f0-9]{4}-){3}[A-Fa-f0-9]{12}"#);
        grok.insert_definition("MAC", r#"(?:%{CISCOMAC}|%{WINDOWSMAC}|%{COMMONMAC})"#);
        grok.insert_definition("CISCOMAC", r#"(?:(?:[A-Fa-f0-9]{4}\.){2}[A-Fa-f0-9]{4})"#);
        grok.insert_definition("WINDOWSMAC", r#"(?:(?:[A-Fa-f0-9]{2}-){5}[A-Fa-f0-9]{2})"#);
        grok.insert_definition("COMMONMAC", r#"(?:(?:[A-Fa-f0-9]{2}:){5}[A-Fa-f0-9]{2})"#);
        grok.insert_definition("IPV6", r#"((([0-9A-Fa-f]{1,4}:){7}([0-9A-Fa-f]{1,4}|:))|(([0-9A-Fa-f]{1,4}:){6}(:[0-9A-Fa-f]{1,4}|((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3})|:))|(([0-9A-Fa-f]{1,4}:){5}(((:[0-9A-Fa-f]{1,4}){1,2})|:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3})|:))|(([0-9A-Fa-f]{1,4}:){4}(((:[0-9A-Fa-f]{1,4}){1,3})|((:[0-9A-Fa-f]{1,4})?:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(([0-9A-Fa-f]{1,4}:){3}(((:[0-9A-Fa-f]{1,4}){1,4})|((:[0-9A-Fa-f]{1,4}){0,2}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(([0-9A-Fa-f]{1,4}:){2}(((:[0-9A-Fa-f]{1,4}){1,5})|((:[0-9A-Fa-f]{1,4}){0,3}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(([0-9A-Fa-f]{1,4}:){1}(((:[0-9A-Fa-f]{1,4}){1,6})|((:[0-9A-Fa-f]{1,4}){0,4}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(:(((:[0-9A-Fa-f]{1,4}){1,7})|((:[0-9A-Fa-f]{1,4}){0,5}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:)))(%.+)?"#);
        grok.insert_definition("IPV4", r#"(?<![0-9])(?:(?:25[0-5]|2[0-4][0-9]|[0-1]?[0-9]{1,2})[.](?:25[0-5]|2[0-4][0-9]|[0-1]?[0-9]{1,2})[.](?:25[0-5]|2[0-4][0-9]|[0-1]?[0-9]{1,2})[.](?:25[0-5]|2[0-4][0-9]|[0-1]?[0-9]{1,2}))(?![0-9])"#);
        grok.insert_definition("IP", r#"(?:%{IPV6}|%{IPV4})"#);
        grok.insert_definition("HOSTNAME", r#"\b(?:[0-9A-Za-z][0-9A-Za-z-]{0,62})(?:\.(?:[0-9A-Za-z][0-9A-Za-z-]{0,62}))*(\.?|\b)"#);
        grok.insert_definition("HOST", r#"%{HOSTNAME}"#);
        grok.insert_definition("IPORHOST", r#"(?:%{HOSTNAME}|%{IP})"#);
        grok.insert_definition("HOSTPORT", r#"%{IPORHOST}:%{POSINT}"#);
        grok.insert_definition("PATH", r#"(?:%{UNIXPATH}|%{WINPATH})"#);
        grok.insert_definition("UNIXPATH", r#"(?>/(?>[\w_%!$@:.,-]+|\\.)*)+"#);
        grok.insert_definition("TTY", r#"(?:/dev/(pts|tty([pq])?)(\w+)?/?(?:[0-9]+))"#);
        grok.insert_definition("WINPATH", r#"(?>[A-Za-z]+:|\\)(?:\\[^\\?*]*)+"#);
        grok.insert_definition("URIPROTO", r#"[A-Za-z]+(\+[A-Za-z+]+)?"#);
        grok.insert_definition("URIHOST", r#"%{IPORHOST}(?::%{POSINT:port})?"#);
        grok.insert_definition("URIPATH", r#"(?:/[A-Za-z0-9$.+!*'(){},~:;=@#%_\-]*)+"#);
        grok.insert_definition("URIPARAM", r#"\?[A-Za-z0-9$.+!*'|(){},~@#%&/=:;_?\-\[\]]*"#);
        grok.insert_definition("URIPATHPARAM", r#"%{URIPATH}(?:%{URIPARAM})?"#);
        grok.insert_definition("URI", r#"%{URIPROTO}://(?:%{USER}(?::[^@]*)?@)?(?:%{URIHOST})?(?:%{URIPATHPARAM})?"#);
        grok.insert_definition("MONTH", r#"\b(?:Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:tember)?|Oct(?:ober)?|Nov(?:ember)?|Dec(?:ember)?)\b"#);
        grok.insert_definition("MONTHNUM", r#"(?:0?[1-9]|1[0-2])"#);
        grok.insert_definition("MONTHNUM2", r#"(?:0[1-9]|1[0-2])"#);
        grok.insert_definition("MONTHDAY", r#"(?:(?:0[1-9])|(?:[12][0-9])|(?:3[01])|[1-9])"#);
        grok.insert_definition("DAY", r#"(?:Mon(?:day)?|Tue(?:sday)?|Wed(?:nesday)?|Thu(?:rsday)?|Fri(?:day)?|Sat(?:urday)?|Sun(?:day)?)"#);
        grok.insert_definition("YEAR", r#"(?>\d\d){1,2}"#);
        grok.insert_definition("HOUR", r#"(?:2[0123]|[01]?[0-9])"#);
        grok.insert_definition("MINUTE", r#"(?:[0-5][0-9])"#);
        grok.insert_definition("SECOND", r#"(?:(?:[0-5]?[0-9]|60)(?:[:.,][0-9]+)?)"#);
        grok.insert_definition("TIME", r#"(?!<[0-9])%{HOUR}:%{MINUTE}(?::%{SECOND})(?![0-9])"#);
        grok.insert_definition("DATE_US", r#"%{MONTHNUM}[/-]%{MONTHDAY}[/-]%{YEAR}"#);
        grok.insert_definition("DATE_EU", r#"%{MONTHDAY}[./-]%{MONTHNUM}[./-]%{YEAR}"#);
        grok.insert_definition("ISO8601_TIMEZONE", r#"(?:Z|[+-]%{HOUR}(?::?%{MINUTE}))"#);
        grok.insert_definition("ISO8601_SECOND", r#"(?:%{SECOND}|60)"#);
        grok.insert_definition("TIMESTAMP_ISO8601", r#"%{YEAR}-%{MONTHNUM}-%{MONTHDAY}[T ]%{HOUR}:?%{MINUTE}(?::?%{SECOND})?%{ISO8601_TIMEZONE}?"#);
        grok.insert_definition("DATE", r#"%{DATE_US}|%{DATE_EU}"#);
        grok.insert_definition("DATESTAMP", r#"%{DATE}[- ]%{TIME}"#);
        grok.insert_definition("TZ", r#"(?:[PMCE][SD]T|UTC)"#);
        grok.insert_definition("DATESTAMP_RFC822", r#"%{DAY} %{MONTH} %{MONTHDAY} %{YEAR} %{TIME} %{TZ}"#);
        grok.insert_definition("DATESTAMP_RFC2822", r#"%{DAY}, %{MONTHDAY} %{MONTH} %{YEAR} %{TIME} %{ISO8601_TIMEZONE}"#);
        grok.insert_definition("DATESTAMP_OTHER", r#"%{DAY} %{MONTH} %{MONTHDAY} %{TIME} %{TZ} %{YEAR}"#);
        grok.insert_definition("DATESTAMP_EVENTLOG", r#"%{YEAR}%{MONTHNUM2}%{MONTHDAY}%{HOUR}%{MINUTE}%{SECOND}"#);
        grok.insert_definition("SYSLOGTIMESTAMP", r#"%{MONTH} +%{MONTHDAY} %{TIME}"#);
        grok.insert_definition("PROG", r#"(?:[\w._/%-]+)"#);
        grok.insert_definition("SYSLOGPROG", r#"%{PROG:program}(?:\[%{POSINT:pid}\])?"#);
        grok.insert_definition("SYSLOGHOST", r#"%{IPORHOST}"#);
        grok.insert_definition("SYSLOGFACILITY", r#"<%{NONNEGINT:facility}.%{NONNEGINT:priority}>"#);
        grok.insert_definition("HTTPDATE", r#"%{MONTHDAY}/%{MONTH}/%{YEAR}:%{TIME} %{INT}"#);
        grok.insert_definition("QS", r#"%{QUOTEDSTRING}"#);
        grok.insert_definition("SYSLOGBASE", r#"%{SYSLOGTIMESTAMP:timestamp} (?:%{SYSLOGFACILITY} )?%{SYSLOGHOST:logsource} %{SYSLOGPROG}:"#);
        grok.insert_definition("COMMONAPACHELOG", r#"%{IPORHOST:clientip} %{USER:ident} %{USER:auth} \[%{HTTPDATE:timestamp}\] "(?:%{WORD:verb} %{NOTSPACE:request}(?: HTTP/%{NUMBER:httpversion})?|%{DATA:rawrequest})" %{NUMBER:response} (?:%{NUMBER:bytes}|-)"#);
        grok.insert_definition("COMBINEDAPACHELOG", r#"%{COMMONAPACHELOG} %{QS:referrer} %{QS:agent}"#);
        grok.insert_definition("LOGLEVEL", r#"([Aa]lert|ALERT|[Tt]race|TRACE|[Dd]ebug|DEBUG|[Nn]otice|NOTICE|[Ii]nfo|INFO|[Ww]arn?(?:ing)?|WARN?(?:ING)?|[Ee]rr?(?:or)?|ERR?(?:OR)?|[Cc]rit?(?:ical)?|CRIT?(?:ICAL)?|[Ff]atal|FATAL|[Ss]evere|SEVERE|EMERG(?:ENCY)?|[Ee]merg(?:ency)?)"#);

        let pattern = match grok.compile(self.pattern.as_str(), false) {
            Ok(p) => p,
            Err(err) => return Err(anyhow!("error compiling grok '{}'", err.to_string())),
        };

        Ok(pattern)
    }
}

impl fmt::Display for Grok_ {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Grok on field: '{}', pattern '{}'", self.modifier.field, self.pattern)
    }
}