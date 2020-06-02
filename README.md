# AWS Parameter Update Tool

This is a small tool used to quickly update simple AWS Parameters.

## CLI Tool

Run `apu -h` to see a list of commands and their usage.

# Updating From a File

The file structure for updating paramters is as follows:
```yaml
- name: "new_parameter"
  value: "Example parameter"
  description: "An example of an unsecure parameter"
  is_secure: false
- name: "new_secure_parameter"
  value: "$uper$ecretP@$$W0rd"
  description: "An example of a secure parameter"
  is_secure: true
```
