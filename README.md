# Browser Backup Tool

一个 Linux 优先的浏览器 Profile 备份 TUI 工具。

当前版本支持扫描 Chromium 系浏览器的 Profile，并通过终端界面生成备份包。

## 当前支持

- Google Chrome
- Chromium
- Brave
- Microsoft Edge

当前只扫描 Linux 下常见的浏览器配置目录，例如：

- `~/.config/google-chrome`
- `~/.config/chromium`
- `~/.config/BraveSoftware/Brave-Browser`
- `~/.config/microsoft-edge`

## 安装和运行

需要先安装 Rust 工具链。

从源码运行：

```bash
git clone https://github.com/xiaoancute/browser_backup_tool.git
cd browser_backup_tool
./run.sh
```

如果想用 release 模式运行：

```bash
./run.sh --release
```

也可以直接从 GitHub 安装：

```bash
cargo install --git https://github.com/xiaoancute/browser_backup_tool.git
browser_backup_tool
```

如果已经 clone 了仓库，也可以用脚本安装到本机：

```bash
./install.sh
browser_backup_tool
```

## TUI 按键

- `Tab`: 在浏览器列表和 Profile 列表之间切换焦点
- `↑` / `↓`: 移动当前焦点列表
- `←` / `→`: 快速切换浏览器
- `Enter`: 查看当前 Profile 详情
- `b`: 打开备份确认页
- `r`: 打开恢复入口页
- `Esc`: 返回上一层；在主界面退出
- `q`: 退出

## 如何备份

1. 运行 `./run.sh` 或 `browser_backup_tool`。
2. 默认焦点在浏览器列表，用 `↑` / `↓` 选择浏览器。
3. 按 `Tab` 切到 Profile 列表，再用 `↑` / `↓` 选择 Profile。
4. 按 `b` 进入备份确认页。
5. 确认浏览器已经关闭。
6. 按 `Enter` 开始备份。

如果对应浏览器仍在运行，工具会阻止备份并提示先关闭浏览器。

备份会输出到：

```text
~/browser-backups/
```

每次备份会生成一个新目录，里面包含：

```text
metadata.json
profile.tar.gz
```

`metadata.json` 记录浏览器名称、Profile 名称、原始路径、平台等信息。

`profile.tar.gz` 是当前 Profile 目录的压缩包。

## 注意事项

- 备份前需要关闭对应浏览器。工具会检测常见 Linux 浏览器进程并阻止运行中备份。
- Profile 可能很大，备份会占用较多磁盘空间。
- 密码、Cookie、登录状态等数据可能依赖系统 keyring 或浏览器加密机制，换机器恢复后不一定能直接使用。
- 恢复功能目前还是入口占位，暂未实现真实恢复。

## 开发

运行测试：

```bash
cargo test
```

检查格式：

```bash
cargo fmt --check
```
