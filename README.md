# Magic Database
Magic database allows you to create your own multiple-connect cache-based databases. You might be thinking about what is reflection protocol.
When you download magic you dont have to install any other stuffs for management or deploying. Only thing that you do is set your `magic.toml` with `magic setup` and start magic server with `magic start`, other alises is below:

- `magic start port/-p 7979 protocol/-r tcp`
- `magic start --port/-p 7979 --protocol/-r udp`
- `magic start --port/-p 7979 --protocol/-r reflection`

With reflection protocol, you can connect multiple magic databases.

Magic is not available to use right now, currently being developed by myself.

## magic.toml
```toml
[server]
port = 7070
bind_address = "127.0.0.1"
protocol = "reflect" # tcp, udp, reflect

[reflect]
targets = ["127.0.0.1:7878", "127.0.0.1:7979"]
```

Get more information with our [website](https://magic.magnesify.com)