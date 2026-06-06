Output docs/assets/patholog-demo.gif

Require cargo

Set Shell "bash"
Set Width 1840
Set Height 1240
Set Padding 48
Set FontSize 30
Set Framerate 24
Set TypingSpeed 1ms

Hide
Type "PROMPT_COMMAND=; PS1=$'\\[\\033[32m\\]~\\[\\033[0m\\] \\[\\033[36m\\]$\\[\\033[0m\\] '"
Enter
Type "cargo build --release --quiet"
Enter
Sleep 2s
Type "patholog_bin=$PWD/target/release/patholog"
Enter
Type "demo_dir=$(mktemp -d)"
Enter
Type "cd $demo_dir"
Enter
Type "mkdir -p project/bin python-shims"
Enter
Type "cat > python-shims/python <<'EOF'"
Enter
Type "#!/usr/bin/env sh"
Enter
Type "printf 'demo-python\\n'"
Enter
Type "EOF"
Enter
Type "chmod 755 python-shims/python"
Enter
Type "printf 'not a directory\\n' > not-a-dir"
Enter
Type "demo_path='project/bin:old-tool:not-a-dir:project/bin:python-shims:/usr/bin:/bin'"
Enter
Type "patholog() { PATH=$demo_path $patholog_bin $@; code=$?; [ $code = 3 ] && return 0; return $code; }"
Enter
Type "demo_comment_index=0"
Enter
Type "demo_comment_prompt() {"
Enter
Type "  case $demo_comment_index in"
Enter
Type "    0) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# diagnose duplicate, missing, and invalid PATH entries' ;;"
Enter
Type "    1) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# explain which python executable wins' ;;"
Enter
Type "    2) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# explain why poetry is not available' ;;"
Enter
Type "    3) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# summarize PATH health for automation' ;;"
Enter
Type "    4) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# preview the managed shell profile block without writing' ;;"
Enter
Type "    *) return ;;"
Enter
Type "  esac"
Enter
Type "  demo_comment_index=$((demo_comment_index + 1))"
Enter
Type "}"
Enter
Type "clear; PROMPT_COMMAND=demo_comment_prompt"
Enter
Sleep 1s
Set TypingSpeed 90ms
Show
Sleep 1600ms

Type "patholog doctor"
Enter
Sleep 1800ms

Type "patholog why python"
Enter
Sleep 1800ms

Type "patholog why-not poetry"
Enter
Sleep 2600ms

Type "patholog health"
Enter
Sleep 1800ms

Type "patholog apply --dry-run --shell zsh --profile .zshrc --drop old-tool"
Enter
Sleep 2600ms
