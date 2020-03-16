use std::fmt;

use grok::{Grok, Pattern};
use serde::export::Formatter;
use serde_json::{Map, Value};

use crate::channels::error::Error;
use crate::channels::mutators::Mutator;
use crate::channels::mutators::*;

#[derive(Debug)]
pub struct Grok_ {
    pub modifier: Mutation,
    compiled_pattern: Pattern,
}

impl Grok_ {
    pub fn new(field: String, pattern: String, custom: Option<&Vec<Value>>) -> Option<Self> {
        let custom_patterns = custom.map(|vec| {
            vec.iter()
                .filter_map(|v| {
                    match v {
                        Value::String(ar) => Some(ar.clone()),
                        _ => {
                            log::error!("all custom values must be a string");
                            None
                        }
                    }
                })
                .filter_map(|raw_pattern| {
                    let index = &raw_pattern.find('.')?;
                    let (a, b) = raw_pattern.split_at(*index);
                    Some((a.to_string(), b.to_string()))
                })
                .collect::<Vec<(String, String)>>()
        });

        let compiled = Grok_::compile(pattern, custom_patterns).ok()?;

        Some(Grok_ {
            modifier: Mutation { field },
            compiled_pattern: compiled,
        })
    }

    pub fn mutate_plain_string(&self, input_data: &[u8]) -> Option<Value> {
        let input = std::str::from_utf8(input_data).ok()?; //TODO Don't ignore this error
        let matches = match self.compiled_pattern.match_against(input) {
            Some(value) => value,
            None => return None,
        };

        let mut values: Vec<(String, Value)> = Vec::new();

        for (x, y) in matches.iter() {
            let k = x;
            let val = y;
            values.push((k.to_string(), Value::from(val)))
        }

        let mut v: Map<String, Value> = Map::new();

        for (x, y) in values.into_iter() {
            v.insert(x, y);
        }

        Some(serde_json::Value::Object(v))
    }

    fn compile(
        pattern: String,
        custom_patterns: Option<Vec<(String, String)>>,
    ) -> Result<Pattern, Error> {
        let mut grok = Grok::default();

        if let Some(custom) = custom_patterns {
            for (a, b) in custom {
                grok.insert_definition(a, b)
            }
        }

        grok.insert_definition("USERNAME", r#"[a-zA-Z0-9._-]+"#);
        grok.insert_definition("USER", r#"%{USERNAME}"#);
        grok.insert_definition("INT", r#"(?:[+-]?(?:[0-9]+))"#);
        grok.insert_definition(
            "BASE10NUM",
            r#"(?<![0-9.+-])(?>[+-]?(?:(?:[0-9]+(?:\.[0-9]+)?)|(?:\.[0-9]+)))"#,
        );
        grok.insert_definition("NUMBER", r#"(?:%{BASE10NUM})"#);
        grok.insert_definition(
            "BASE16NUM",
            r#"(?<![0-9A-Fa-f])(?:[+-]?(?:0x)?(?:[0-9A-Fa-f]+))"#,
        );
        grok.insert_definition("BASE16FLOAT", r#"\b(?<![0-9A-Fa-f.])(?:[+-]?(?:0x)?(?:(?:[0-9A-Fa-f]+(?:\.[0-9A-Fa-f]*)?)|(?:\.[0-9A-Fa-f]+)))\b"#);
        grok.insert_definition("POSINT", r#"\b(?:[1-9][0-9]*)\b"#);
        grok.insert_definition("NONNEGINT", r#"\b(?:[0-9]+)\b"#);
        grok.insert_definition("WORD", r#"\b\w+\b"#);
        grok.insert_definition("NOTSPACE", r#"\S+"#);
        grok.insert_definition("SPACE", r#"\s*"#);
        grok.insert_definition("DATA", r#".*?"#);
        grok.insert_definition("GREEDYDATA", r#".*"#);
        grok.insert_definition("QUOTEDSTRING", r#"(?>(?<!\\)(?>"(?>\\.|[^\\"]+)+"|""|(?>'(?>\\.|[^\\']+)+')|''|(?>`(?>\\.|[^\\`]+)+`)|``))"#);
        grok.insert_definition(
            "UUID",
            r#"[A-Fa-f0-9]{8}-(?:[A-Fa-f0-9]{4}-){3}[A-Fa-f0-9]{12}"#,
        );
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
        grok.insert_definition(
            "URI",
            r#"%{URIPROTO}://(?:%{USER}(?::[^@]*)?@)?(?:%{URIHOST})?(?:%{URIPATHPARAM})?"#,
        );
        grok.insert_definition("MONTH", r#"\b(?:Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:tember)?|Oct(?:ober)?|Nov(?:ember)?|Dec(?:ember)?)\b"#);
        grok.insert_definition("MONTHNUM", r#"(?:0?[1-9]|1[0-2])"#);
        grok.insert_definition("MONTHNUM2", r#"(?:0[1-9]|1[0-2])"#);
        grok.insert_definition(
            "MONTHDAY",
            r#"(?:(?:0[1-9])|(?:[12][0-9])|(?:3[01])|[1-9])"#,
        );
        grok.insert_definition("DAY", r#"(?:Mon(?:day)?|Tue(?:sday)?|Wed(?:nesday)?|Thu(?:rsday)?|Fri(?:day)?|Sat(?:urday)?|Sun(?:day)?)"#);
        grok.insert_definition("YEAR", r#"(?>\d\d){1,2}"#);
        grok.insert_definition("HOUR", r#"(?:2[0123]|[01]?[0-9])"#);
        grok.insert_definition("MINUTE", r#"(?:[0-5][0-9])"#);
        grok.insert_definition("SECOND", r#"(?:(?:[0-5]?[0-9]|60)(?:[:.,][0-9]+)?)"#);
        grok.insert_definition(
            "TIME",
            r#"(?!<[0-9])%{HOUR}:%{MINUTE}(?::%{SECOND})(?![0-9])"#,
        );
        grok.insert_definition("DATE_US", r#"%{MONTHNUM}[/-]%{MONTHDAY}[/-]%{YEAR}"#);
        grok.insert_definition("DATE_EU", r#"%{MONTHDAY}[./-]%{MONTHNUM}[./-]%{YEAR}"#);
        grok.insert_definition("ISO8601_TIMEZONE", r#"(?:Z|[+-]%{HOUR}(?::?%{MINUTE}))"#);
        grok.insert_definition("ISO8601_SECOND", r#"(?:%{SECOND}|60)"#);
        grok.insert_definition("TIMESTAMP_ISO8601", r#"%{YEAR}-%{MONTHNUM}-%{MONTHDAY}[T ]%{HOUR}:?%{MINUTE}(?::?%{SECOND})?%{ISO8601_TIMEZONE}?"#);
        grok.insert_definition("DATE", r#"%{DATE_US}|%{DATE_EU}"#);
        grok.insert_definition("DATESTAMP", r#"%{DATE}[- ]%{TIME}"#);
        grok.insert_definition("TZ", r#"(?:[PMCE][SD]T|UTC)"#);
        grok.insert_definition(
            "DATESTAMP_RFC822",
            r#"%{DAY} %{MONTH} %{MONTHDAY} %{YEAR} %{TIME} %{TZ}"#,
        );
        grok.insert_definition(
            "DATESTAMP_RFC2822",
            r#"%{DAY}, %{MONTHDAY} %{MONTH} %{YEAR} %{TIME} %{ISO8601_TIMEZONE}"#,
        );
        grok.insert_definition(
            "DATESTAMP_OTHER",
            r#"%{DAY} %{MONTH} %{MONTHDAY} %{TIME} %{TZ} %{YEAR}"#,
        );
        grok.insert_definition(
            "DATESTAMP_EVENTLOG",
            r#"%{YEAR}%{MONTHNUM2}%{MONTHDAY}%{HOUR}%{MINUTE}%{SECOND}"#,
        );
        grok.insert_definition("SYSLOGTIMESTAMP", r#"%{MONTH} +%{MONTHDAY} %{TIME}"#);
        grok.insert_definition("PROG", r#"(?:[\w._/%-]+)"#);
        grok.insert_definition("SYSLOGPROG", r#"%{PROG:program}(?:\[%{POSINT:pid}\])?"#);
        grok.insert_definition("SYSLOGHOST", r#"%{IPORHOST}"#);
        grok.insert_definition(
            "SYSLOGFACILITY",
            r#"<%{NONNEGINT:facility}.%{NONNEGINT:priority}>"#,
        );
        grok.insert_definition("HTTPDATE", r#"%{MONTHDAY}/%{MONTH}/%{YEAR}:%{TIME} %{INT}"#);
        grok.insert_definition("QS", r#"%{QUOTEDSTRING}"#);
        grok.insert_definition("SYSLOGBASE", r#"%{SYSLOGTIMESTAMP:timestamp} (?:%{SYSLOGFACILITY} )?%{SYSLOGHOST:logsource} %{SYSLOGPROG}:"#);
        grok.insert_definition("COMMONAPACHELOG", r#"%{IPORHOST:clientip} %{USER:ident} %{USER:auth} \[%{HTTPDATE:timestamp}\] "(?:%{WORD:verb} %{NOTSPACE:request}(?: HTTP/%{NUMBER:httpversion})?|%{DATA:rawrequest})" %{NUMBER:response} (?:%{NUMBER:bytes}|-)"#);
        grok.insert_definition(
            "COMBINEDAPACHELOG",
            r#"%{COMMONAPACHELOG} %{QS:referrer} %{QS:agent}"#,
        );
        grok.insert_definition("LOGLEVEL", r#"([Aa]lert|ALERT|[Tt]race|TRACE|[Dd]ebug|DEBUG|[Nn]otice|NOTICE|[Ii]nfo|INFO|[Ww]arn?(?:ing)?|WARN?(?:ING)?|[Ee]rr?(?:or)?|ERR?(?:OR)?|[Cc]rit?(?:ical)?|CRIT?(?:ICAL)?|[Ff]atal|FATAL|[Ss]evere|SEVERE|EMERG(?:ENCY)?|[Ee]merg(?:ency)?)"#);

        let pattern = grok
            .compile(pattern.as_str(), false)
            .map_err(Error::GrokError)?;

        Ok(pattern)
    }
}

impl Mutator for Grok_ {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error> {
        let value = v
            .get(&self.modifier.field)
            .ok_or_else(|| Error::FieldNotFoundInJSON(self.modifier.field.clone()))?;

        let incoming_value = match value {
            Value::String(s) => s,
            _ => return Error::NotString(value.to_string()).into(),
        };

        let matches = self
            .compiled_pattern
            .match_against(incoming_value.as_str())
            .ok_or(Error::GrokNoMatches)?;

        let mut values: Vec<(String, Value)> = Vec::new();

        for (x, y) in matches.iter() {
            let k = x;
            let val = y;
            if !val.is_empty() {
                values.push((k.to_string(), Value::from(val)))
            }
        }

        for (x, y) in values.into_iter() {
            v.insert(x, y);
        }

        Ok(())
    }

    fn mutator_type(&self) -> MutatorType { MutatorType::Grok }

    fn as_grok(&self) -> Option<&Grok_> { Some(self) }
}

impl fmt::Display for Grok_ {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Grok on field: '{}'", self.modifier.field)
    }
}
