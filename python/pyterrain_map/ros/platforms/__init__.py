"""
Platform-specific configurations for PyTerrainMap ROS bridge.

Pre-configured sensor and TF setup for common robot platforms.
"""

from typing import Dict, Any
import yaml


# Boston Dynamics Spot
SPOT_CONFIG = {
    "robot_id": "spot_1",
    "robot_type": "quadruped",
    "base_frame": "body",
    "reference_frame": "map",
    "sensors": {
        "lidar": {
            "enabled": True,
            "topic": "/scan",
            "frame_id": "lidar",
            "adapter": "lidar",
            "params": {
                "voxel_size_m": 0.1,
                "min_range_m": 0.1,
                "max_range_m": 20.0,
            },
        },
        "rgb": {
            "enabled": True,
            "topics": [
                "/camera/frontleft/image_raw",
                "/camera/frontright/image_raw",
                "/camera/back/image_raw",
            ],
            "frame_ids": ["frontleft_rgb", "frontright_rgb", "back_rgb"],
            "adapter": "rgb",
        },
        "imu": {
            "enabled": True,
            "topic": "/imu/data",
            "frame_id": "imu_link",
            "adapter": "imu",
        },
    },
    "tfs": {
        "static": [
            {"parent": "body", "child": "lidar", "xyz": [0, 0, 0.3], "rpy": [0, 0, 0]},
            {"parent": "body", "child": "frontleft_rgb", "xyz": [0.2, 0.1, 0.2], "rpy": [0, 0, 0]},
            {"parent": "body", "child": "imu_link", "xyz": [0, 0, 0], "rpy": [0, 0, 0]},
        ]
    },
    "backend": {
        "type": "local",
        "config": {"base_path": "~/.pyterrain/spot_observations"},
    },
}

# DJI M300 RTK
DJI_M300_CONFIG = {
    "robot_id": "m300_1",
    "robot_type": "quadrotor",
    "base_frame": "base_link",
    "reference_frame": "map",  # RTK provides global coordinates
    "coordinate_mode": "geodetic",  # Direct GPS/RTK
    "sensors": {
        "lidar": {
            "enabled": True,
            "topic": "/lidar_points",
            "frame_id": "lidar_frame",
            "adapter": "lidar",
            "params": {
                "voxel_size_m": 0.05,  # Finer resolution for aerial
                "min_range_m": 0.1,
                "max_range_m": 120.0,
            },
        },
        "thermal": {
            "enabled": True,
            "topic": "/zenmuse_h20t/thermal/image_raw",
            "frame_id": "thermal_frame",
            "adapter": "thermal",
            "params": {
                "grid_size": 16,
                "min_temp": -40.0,
                "max_temp": 85.0,
            },
        },
        "rgb": {
            "enabled": True,
            "topic": "/zenmuse_h20t/rgb/image_raw",
            "frame_id": "rgb_frame",
            "adapter": "rgb",
        },
    },
    "tfs": {
        "static": [
            {"parent": "base_link", "child": "lidar_frame", "xyz": [0, 0, 0.2], "rpy": [0, 0, 0]},
            {"parent": "base_link", "child": "thermal_frame", "xyz": [0.1, 0, 0], "rpy": [0, 0, 0]},
            {"parent": "base_link", "child": "rgb_frame", "xyz": [0.1, 0, 0], "rpy": [0, 0, 0]},
        ]
    },
    "backend": {
        "type": "s3",
        "config": {
            "bucket": "pyterrain-dji",
            "prefix": "m300_observations",
            "region": "us-east-1",
        },
    },
}

# Clearpath Warthog
WARTHOG_CONFIG = {
    "robot_id": "warthog_1",
    "robot_type": "ugv",
    "base_frame": "base_link",
    "reference_frame": "map",
    "sensors": {
        "lidar": {
            "enabled": True,
            "topic": "/lidar/scan",
            "frame_id": "lidar_link",
            "adapter": "lidar",
            "params": {
                "voxel_size_m": 0.1,
                "min_range_m": 0.1,
                "max_range_m": 50.0,
            },
        },
        "front_camera": {
            "enabled": True,
            "topic": "/camera/front/image_raw",
            "frame_id": "camera_link",
            "adapter": "rgb",
        },
        "gps": {
            "enabled": True,
            "topic": "/gps/fix",
            "frame_id": "gps_link",
            "adapter": "gps",
        },
        "imu": {
            "enabled": True,
            "topic": "/imu/data",
            "frame_id": "imu_link",
            "adapter": "imu",
        },
    },
    "tfs": {
        "static": [
            {"parent": "base_link", "child": "lidar_link", "xyz": [0, 0, 0.5], "rpy": [0, 0, 0]},
            {"parent": "base_link", "child": "camera_link", "xyz": [0.3, 0, 0.3], "rpy": [0, 0.3, 0]},
            {"parent": "base_link", "child": "imu_link", "xyz": [0, 0, 0], "rpy": [0, 0, 0]},
        ]
    },
    "backend": {
        "type": "local",
        "config": {"base_path": "~/.pyterrain/warthog_observations"},
    },
}

# Generic template for custom robots
GENERIC_CONFIG = {
    "robot_id": "robot_1",
    "robot_type": "custom",
    "base_frame": "base_link",
    "reference_frame": "map",
    "sensors": {
        # Add your sensors here
    },
    "tfs": {
        "static": [
            # Add static TF relationships here
        ]
    },
    "backend": {
        "type": "local",
        "config": {"base_path": "~/.pyterrain/observations"},
    },
}

# Platform registry
PLATFORMS = {
    "spot": SPOT_CONFIG,
    "dji_m300": DJI_M300_CONFIG,
    "warthog": WARTHOG_CONFIG,
    "generic": GENERIC_CONFIG,
}


def get_platform_config(platform_name: str) -> Dict[str, Any]:
    """
    Get configuration for a platform.

    Args:
        platform_name: "spot", "dji_m300", "warthog", "generic"

    Returns:
        Configuration dictionary
    """
    if platform_name not in PLATFORMS:
        raise ValueError(f"Unknown platform: {platform_name}. Available: {list(PLATFORMS.keys())}")
    return PLATFORMS[platform_name].copy()


def save_platform_config(config: Dict[str, Any], filepath: str):
    """Save platform configuration to YAML file."""
    with open(filepath, "w") as f:
        yaml.dump(config, f, default_flow_style=False)


def load_platform_config(filepath: str) -> Dict[str, Any]:
    """Load platform configuration from YAML file."""
    with open(filepath) as f:
        return yaml.safe_load(f)


__all__ = [
    "SPOT_CONFIG",
    "DJI_M300_CONFIG",
    "WARTHOG_CONFIG",
    "GENERIC_CONFIG",
    "PLATFORMS",
    "get_platform_config",
    "save_platform_config",
    "load_platform_config",
]
