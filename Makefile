# Learna 任务入口：脚本统一封装，Make 只做转发
.PHONY: check test setup doctor hooks changelog db-migrate db-status db-reset db-backup

check:            ## 统一质量检查（格式/静态/类型）
	bash scripts/check/check.sh
test:             ## 三栈测试
	bash scripts/test/test.sh
setup:            ## 开发环境初始化（dev 模式；docker 模式: make setup MODE=docker）
	bash scripts/setup.sh $(MODE)
doctor:           ## 环境体检
	bash scripts/dev/doctor.sh
hooks:            ## 安装 .githooks
	bash scripts/git/install-hooks.sh

db-migrate:       ## 执行数据库迁移
	bash scripts/db/migrate.sh
db-status:        ## 检查迁移状态
	bash scripts/db/status.sh
db-reset:         ## 重建本地开发数据库（危险，需确认）
	bash scripts/db/reset.sh
db-backup:        ## 备份本地数据库
	bash scripts/db/backup.sh

changelog:        ## 生成/更新 CHANGELOG.md
	bash scripts/release/changelog.sh
