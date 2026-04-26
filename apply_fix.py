import sys

file_path = 'agent-rs/src/main.rs'
with open(file_path, 'r') as f:
    lines = f.readlines()

new_lines = []
i = 0
while i < len(lines):
    line = lines[i]
    # Match first occurrence
    if 'let output = std::process::Command::new("gsd-sdk")' in line and i + 1 < len(lines) and '.arg(sdk_cmd)' in lines[i+1] and '.output()' in lines[i+2]:
        new_lines.append(line)
        new_lines.append(lines[i+1])
        new_lines.append('                                        .arg("--model")\n')
        new_lines.append('                                        .arg(&app_config.model_name)\n')
        new_lines.append(lines[i+2])
        i += 3
    # Match second occurrence
    elif 'let output = std::process::Command::new("gsd-sdk")' in line and i + 2 < len(lines) and '.arg(sdk_cmd)' in lines[i+1] and '.args(&args)' in lines[i+2] and '.output()' in lines[i+3]:
        new_lines.append(line)
        new_lines.append(lines[i+1])
        new_lines.append(lines[i+2])
        new_lines.append('                                        .arg("--model")\n')
        new_lines.append('                                        .arg(&app_config.model_name)\n')
        new_lines.append(lines[i+3])
        i += 4
    else:
        new_lines.append(line)
        i += 1

with open(file_path, 'w') as f:
    f.writelines(new_lines)
