"""配置测试"""

from pathlib import Path

from qtcloud_finance_cli.config import Settings


def test_settings_defaults():
    """测试默认配置"""
    settings = Settings()

    assert settings.ollama_host == "http://localhost:11434"
    assert settings.ollama_model == "qwen2.5-coder:3b"
    assert settings.temperature == 0.1
    assert settings.max_tokens == 1000
    assert isinstance(settings.data_root, Path)


def test_settings_custom_values(tmp_data_dir):
    """测试自定义配置"""
    settings = Settings(
        ollama_host="http://192.168.1.100:11434",
        ollama_model="qwen2.5:7b",
        data_root=tmp_data_dir,
        temperature=0.5,
        max_tokens=2000,
    )

    assert settings.ollama_host == "http://192.168.1.100:11434"
    assert settings.ollama_model == "qwen2.5:7b"
    assert settings.data_root == tmp_data_dir
    assert settings.temperature == 0.5
    assert settings.max_tokens == 2000
