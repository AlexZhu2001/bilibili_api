# bilibili_api

![GitHub License](https://img.shields.io/github/license/AlexZhu2001/bilibili_api)

本项目是基于[哔哩哔哩-API收集整理](https://github.com/SocialSisterYi/bilibili-API-collect)的Rust实现

## 项目内容
本项目包含了部分常用的主站业务接口，主要包含以下的API（包含勾的为已经实现）
- [] Wbi签名的获取及签名(WbiClient)
- [] 登录
    - [x] 二维码登录 （部分测试）
    - [x] Cookie刷新（未测试）
- [] 账户信息
    - [x] 个人中心-我的信息
    - [x] 个人中心-大会员信息
    - [] 导航栏个人信息
- [] 稿件相关
    - [] 稿件基本信息
    - [] 稿件取流 （不保证特殊视频，例如互动视频的取流）
    - [] 稿件弹幕获取（不包含BAS弹幕）
    - [] 评论获取
- [] 收藏夹信息
- [] 搜索功能

剩余常用API将会陆续实现，但不保证实现也没有意愿实现全部的API

*由于隐私和版权等相关的原因，部分API未经过完全测试甚至未测试，请谨慎使用*

## 温馨提示
<strong>
<h3 style="color: #cc1111;">本项目遵守 [MIT](LICENSE) 协议，请勿滥用本项目，使用本项目造成的任何不良影响及后果自行承担</h3>
</strong>

## 鸣谢
哔哩哔哩-API收集整理](https://github.com/SocialSisterYi/bilibili-API-collect)
