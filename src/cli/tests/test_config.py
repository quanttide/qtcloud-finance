"""配置测试"""

from pathlib import Path

from qtcloud_finance_cli.config import Settings


def test_settings_defaults():
    """测试默认配置"""
    settings = Settings()

    assert settings.llm_model == "deepseek-v4-flash"
    assert settings.llm_base_url == "https://api.deepseek.com"
    assert settings.temperature == 0.1
    assert settings.max_tokens == 1000
    assert isinstance(settings.data_root, Path)


def test_settings_custom_values(tmp_data_dir):
    """测试自定义配置"""
    settings = Settings(
        llm_model="deepseek-v4-pro",
        llm_base_url="https://custom.deepseek.com",
        llm_api_key="sk-test",
        data_root=tmp_data_dir,
        temperature=0.5,
        max_tokens=2000,
    )

    assert settings.llm_model == "deepseek-v4-pro"
    assert settings.llm_base_url == "https://custom.deepseek.com"
    assert settings.llm_api_key == "sk-test"
    assert settings.data_root == tmp_data_dir
    assert settings.temperature == 0.5
    assert settings.max_tokens == 2000
