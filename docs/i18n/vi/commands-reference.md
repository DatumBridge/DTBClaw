# Tham khảo lệnh OctoClaw

Dựa trên CLI hiện tại (`octoclaw --help`).

Xác minh lần cuối: **2026-02-28**.

## Lệnh cấp cao nhất

| Lệnh | Mục đích |
|---|---|
| `onboard` | Khởi tạo workspace/config nhanh hoặc tương tác |
| `agent` | Chạy chat tương tác hoặc chế độ gửi tin nhắn đơn |
| `gateway` | Khởi động gateway webhook và HTTP WhatsApp |
| `daemon` | Khởi động runtime có giám sát (gateway + channels + heartbeat/scheduler tùy chọn) |
| `service` | Quản lý vòng đời dịch vụ cấp hệ điều hành |
| `doctor` | Chạy chẩn đoán và kiểm tra trạng thái |
| `status` | Hiển thị cấu hình và tóm tắt hệ thống |
| `cron` | Quản lý tác vụ định kỳ |
| `models` | Làm mới danh mục model của provider |
| `providers` | Liệt kê ID provider, bí danh và provider đang dùng |
| `channel` | Quản lý kênh và kiểm tra sức khỏe kênh |
| `integrations` | Kiểm tra chi tiết tích hợp |
| `skills` | Liệt kê/cài đặt/gỡ bỏ skills |
| `migrate` | Nhập dữ liệu từ runtime khác (hiện hỗ trợ OpenClaw) |
| `config` | Kiểm tra, truy vấn và sửa đổi cấu hình runtime |
| `completions` | Tạo script tự hoàn thành cho shell ra stdout |
| `hardware` | Phát hiện và kiểm tra phần cứng USB |
| `peripheral` | Cấu hình và nạp firmware thiết bị ngoại vi |

## Nhóm lệnh

### `onboard`

- `octoclaw onboard`
- `octoclaw onboard --interactive`
- `octoclaw onboard --channels-only`
- `octoclaw onboard --api-key <KEY> --provider <ID> --memory <sqlite|lucid|markdown|none>`
- `octoclaw onboard --api-key <KEY> --provider <ID> --model <MODEL_ID> --memory <sqlite|lucid|markdown|none>`
- `octoclaw onboard --migrate-openclaw`
- `octoclaw onboard --migrate-openclaw --openclaw-source <PATH> --openclaw-config <PATH>`

### `agent`

- `octoclaw agent`
- `octoclaw agent -m "Hello"`
- `octoclaw agent --provider <ID> --model <MODEL> --temperature <0.0-2.0>`
- `octoclaw agent --peripheral <board:path>`

### `gateway` / `daemon`

- `octoclaw gateway [--host <HOST>] [--port <PORT>] [--new-pairing]`
- `octoclaw daemon [--host <HOST>] [--port <PORT>]`

`--new-pairing` sẽ xóa toàn bộ token đã ghép đôi và tạo mã ghép đôi mới khi gateway khởi động.

### `service`

- `octoclaw service install`
- `octoclaw service start`
- `octoclaw service stop`
- `octoclaw service restart`
- `octoclaw service status`
- `octoclaw service uninstall`

### `cron`

- `octoclaw cron list`
- `octoclaw cron add <expr> [--tz <IANA_TZ>] <command>`
- `octoclaw cron add-at <rfc3339_timestamp> <command>`
- `octoclaw cron add-every <every_ms> <command>`
- `octoclaw cron once <delay> <command>`
- `octoclaw cron remove <id>`
- `octoclaw cron pause <id>`
- `octoclaw cron resume <id>`

### `models`

- `octoclaw models refresh`
- `octoclaw models refresh --provider <ID>`
- `octoclaw models refresh --force`

`models refresh` hiện hỗ trợ làm mới danh mục trực tiếp cho các provider: `openrouter`, `openai`, `anthropic`, `groq`, `mistral`, `deepseek`, `xai`, `together-ai`, `gemini`, `ollama`, `llamacpp`, `sglang`, `vllm`, `astrai`, `venice`, `fireworks`, `cohere`, `moonshot`, `stepfun`, `glm`, `zai`, `qwen`, `volcengine` (alias `doubao`/`ark`), `siliconflow` và `nvidia`.

### `channel`

- `octoclaw channel list`
- `octoclaw channel start`
- `octoclaw channel doctor`
- `octoclaw channel bind-telegram <IDENTITY>`
- `octoclaw channel add <type> <json>`
- `octoclaw channel remove <name>`

Lệnh trong chat khi runtime đang chạy (Telegram/Discord):

- `/models`
- `/models <provider>`
- `/model`
- `/model <model-id>`

Channel runtime cũng theo dõi `config.toml` và tự động áp dụng thay đổi cho:
- `default_provider`
- `default_model`
- `default_temperature`
- `api_key` / `api_url` (cho provider mặc định)
- `reliability.*` cài đặt retry của provider

`add/remove` hiện chuyển hướng về thiết lập có hướng dẫn / cấu hình thủ công (chưa hỗ trợ đầy đủ mutator khai báo).

### `integrations`

- `octoclaw integrations info <name>`

### `skills`

- `octoclaw skills list`
- `octoclaw skills install <source>`
- `octoclaw skills remove <name>`

`<source>` chấp nhận git remote (`https://...`, `http://...`, `ssh://...` và `git@host:owner/repo.git`) hoặc đường dẫn cục bộ.

Skill manifest (`SKILL.toml`) hỗ trợ `prompts` và `[[tools]]`; cả hai được đưa vào system prompt của agent khi chạy, giúp model có thể tuân theo hướng dẫn skill mà không cần đọc thủ công.

### `migrate`

- `octoclaw migrate openclaw [--source <path>] [--source-config <path>] [--dry-run]`

Gợi ý: trong hội thoại agent, bề mặt tool `openclaw_migration` cho phép preview hoặc áp dụng migration bằng tool-call có kiểm soát quyền.

### `config`

- `octoclaw config show`
- `octoclaw config get <key>`
- `octoclaw config set <key> <value>`
- `octoclaw config schema`

`config show` xuất toàn bộ cấu hình hiệu lực dưới dạng JSON với các trường nhạy cảm được ẩn thành `***REDACTED***`. Các ghi đè từ biến môi trường đã được áp dụng.

`config get <key>` truy vấn một giá trị theo đường dẫn phân tách bằng dấu chấm (ví dụ: `gateway.port`, `security.estop.enabled`). Giá trị đơn in trực tiếp; đối tượng và mảng in dạng JSON.

`config set <key> <value>` cập nhật giá trị cấu hình và lưu nguyên tử vào `config.toml`. Kiểu dữ liệu được suy luận tự động (`true`/`false` → bool, số nguyên, số thực, cú pháp JSON → đối tượng/mảng, còn lại → chuỗi). Sai kiểu sẽ bị từ chối trước khi ghi.

`config schema` xuất JSON Schema (draft 2020-12) cho toàn bộ hợp đồng `config.toml` ra stdout.

### `completions`

- `octoclaw completions bash`
- `octoclaw completions fish`
- `octoclaw completions zsh`
- `octoclaw completions powershell`
- `octoclaw completions elvish`

`completions` chỉ xuất ra stdout để script có thể được source trực tiếp mà không bị lẫn log/cảnh báo.

### `hardware`

- `octoclaw hardware discover`
- `octoclaw hardware introspect <path>`
- `octoclaw hardware info [--chip <chip_name>]`

### `peripheral`

- `octoclaw peripheral list`
- `octoclaw peripheral add <board> <path>`
- `octoclaw peripheral flash [--port <serial_port>]`
- `octoclaw peripheral setup-uno-q [--host <ip_or_host>]`
- `octoclaw peripheral flash-nucleo`

## Kiểm tra nhanh

Để xác minh nhanh tài liệu với binary hiện tại:

```bash
octoclaw --help
octoclaw <command> --help
```
