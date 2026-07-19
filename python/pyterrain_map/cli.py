"""PyTerrainMap CLI Interface

Interactive setup, configuration, and terrain analysis.
Accessible via: pytm <command>
"""

import sys
import json
import argparse
from pathlib import Path
from typing import Optional


def setup_command(args):
    """Run interactive setup wizard."""
    from .setup_wizard import SetupWizard

    config_dir = Path(args.config_dir) if args.config_dir else None
    wizard = SetupWizard(config_dir=config_dir)
    success = wizard.run()
    return 0 if success else 1


def configure_command(args):
    """Reconfigure warehouse (update existing setup)."""
    from .setup_wizard import SetupWizard

    config_dir = Path(args.config_dir) if args.config_dir else None
    wizard = SetupWizard(config_dir=config_dir)

    print("\n" + "="*60)
    print("  PyTerrainMap Configuration Update")
    print("="*60 + "\n")

    if wizard.current_config:
        print("Current configuration found:")
        print(f"  Warehouse: {wizard.current_config.get('warehouse', 'unknown')}")
        print()

    success = wizard.run()
    return 0 if success else 1


def version_command(args):
    """Print version information."""
    try:
        from . import __version__
        print(f"PyTerrainMap {__version__}")
    except ImportError:
        print("PyTerrainMap v0.1.0")
    return 0


def query_command(args):
    """Query observations from PyTerrainMap."""
    print("Query command - Not yet implemented")
    return 1


def main() -> int:
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        prog="pytm",
        description="PyTerrainMap: Collaborative terrain mapping for multi-robot fleets",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  pytm setup                      # Initial configuration wizard
  pytm configure                  # Update warehouse configuration
  pytm version                    # Show version information
        """,
    )

    parser.add_argument(
        "--version",
        action="version",
        version="%(prog)s 0.1.0",
    )

    parser.add_argument(
        "--config-dir",
        default=None,
        help="Configuration directory (default: ~/.pyterrain)",
    )

    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # Setup command
    setup_parser = subparsers.add_parser(
        "setup",
        help="Interactive setup wizard for PyTerrainMap (first-time configuration)",
    )
    setup_parser.set_defaults(func=setup_command)

    # Configure command
    configure_parser = subparsers.add_parser(
        "configure",
        help="Reconfigure PyTerrainMap warehouse settings",
    )
    configure_parser.set_defaults(func=configure_command)

    # Version command
    version_parser = subparsers.add_parser(
        "version",
        help="Print version information",
    )
    version_parser.set_defaults(func=version_command)

    # Query command
    query_parser = subparsers.add_parser(
        "query",
        help="Query observations from PyTerrainMap",
    )
    query_parser.set_defaults(func=query_command)

    # If no args provided, show help
    if len(sys.argv) == 1:
        parser.print_help()
        return 0

    args = parser.parse_args()

    if hasattr(args, "func"):
        return args.func(args)
    else:
        parser.print_help()
        return 0


if __name__ == "__main__":
    sys.exit(main())
