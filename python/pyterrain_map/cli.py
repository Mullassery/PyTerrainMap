"""PyTerrainMap CLI Interface

Natural language command interface for terrain analysis.
Accessible via: pytm <command>
"""

import sys
import json
from typing import Optional


def main() -> int:
    """Main CLI entry point."""
    if len(sys.argv) < 2:
        print("PyTerrainMap CLI v0.0.1")
        print("Try: pytm help")
        return 0

    command = " ".join(sys.argv[1:])

    # Import Rust CLI parser
    try:
        from . import _core as rust_core
        cli_command = rust_core.cli.CLICommand.parse(command)
    except ImportError:
        print("Error: Rust extension not loaded")
        return 1
    except Exception as e:
        print(f"Error parsing command: {e}")
        return 1

    # Format output
    if "--json" in command:
        try:
            response_json = cli_command.format_json()
            print(response_json)
        except Exception as e:
            print(json.dumps({"error": str(e)}))
            return 1
    else:
        try:
            formatted = cli_command.format_terminal()
            print(formatted)
        except Exception as e:
            print(f"Error: {e}")
            return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
