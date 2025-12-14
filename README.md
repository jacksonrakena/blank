# bl[ a]nk
Simple link redirector service, and some other small utilities.

### Redirector
"Rules" are expected to be included at runtime, either via environment variable or a config file, using the KDL document format:
```kdl
// Shorthand
site "https://jacksonrakena.com"

// Full
github {
    target "https://github.com/jacksonrakena"
    // Optional fields
    description "My GitHub profile"
    tags "code" "projects"
    mode "permanent"
}
```

Then, serve:
```
cargo run --release
```

And redirect by visiting `http://localhost:3000/{rule_name}`.

The idea here is that you can set it up on a subdomain like `go.company.com`, and then use search domains or something like MagicDNS to 
make `go/xyz` redirect to `go.company.com/xyz`, which then redirects to the full URL.