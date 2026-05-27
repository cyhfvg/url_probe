## url_probe

### function todo

- [y] 用clap 把所有参数定义出来
- [y] -t, --target , url列表或url, 支持 `-t -`从stdin中获取输入
- [y] --filter-http-code ,用来筛选http-code，只输出匹配http-code的结果，设置为空（全部输出）
- [y] --black-http-code ,用来过滤Http-code,不在结果中输出，默认为空
- [y] --black-size ,用来过滤 size，不在结果中输出，默认为空
- [y] --enum-path-dict 不纳入 `url_probe`；路径字典枚举由独立工具实现，本工具不扩展路径发现职责
- [y] --concurrency ,用来设置同时访问的“线程“数
- [y] --timeout , 用来设置单url访问超时时间
- [y] --retry , 用来设置访问超时，访问失败时的重试次数
- [y] -o, --output , 用来设置输出的文件路径，默认为输出到stdout
- [y] --method , 默认GET，可选 HEAD、POST等
- [y] --user-agent , 设置user-agent，默认 `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36`
- [y] --follow-redirect ,默认开启，跟随url跳转
- [y] --insecure , 开放忽略TLS证书错误，默认开启
- [y] --output-with-error , 输出时将失败结果也输出，默认开启
- [y] --format , csv
- [y] --proxy ,以 scheme URL 形式提供代理配置，支持 HTTP/HTTPS 和带认证的 SOCKS5H

```md
实现单个 URL 探测 probe_once()
把结果打印成一行 CSV
加入 Tokio 并发
加入 retry
加入过滤器
加入输出文件
```
