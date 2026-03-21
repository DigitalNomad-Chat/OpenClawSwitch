#!/usr/bin/env python3
"""
Agent Workspaces Manifest 生成脚本
从设计文档生成包含30个岗位的 manifest.json
"""

import json
import os
from pathlib import Path
from datetime import datetime

# 30个岗位的完整元数据
WORKSPACES_DATA = [
    # 一、内容创作类 (6个)
    {
        "id": "copywriting-expert",
        "name": "文案写作专家",
        "description": "专业的营销文案和产品描述撰写",
        "category": "content",
        "recommended": True,
        "recommendedReason": "高频使用场景，适合电商、营销人员",
        "tags": ["写作", "营销", "文案", "转化"],
        "capabilities": ["产品卖点提炼", "痛点场景描述", "转化型文案写作", "A/B测试文案生成"],
        "scenarios": ["商品详情页文案", "广告创意文案", "营销邮件撰写", "社交媒体文案"],
        "dirName": "001.copywriting-expert"
    },
    {
        "id": "social-media-manager",
        "name": "社交媒体运营",
        "description": "多平台内容规划和发布管理",
        "category": "content",
        "recommended": True,
        "recommendedReason": "社交媒体运营必备，覆盖主流平台",
        "tags": ["社交媒体", "运营", "内容", "增长"],
        "capabilities": ["平台特性适配", "发布时间优化", "互动话术设计", "粉丝增长策略"],
        "scenarios": ["小红书内容运营", "抖音短视频脚本", "微信公众号管理", "跨平台内容分发"],
        "dirName": "002.social-media-manager"
    },
    {
        "id": "video-script-writer",
        "name": "视频脚本策划",
        "description": "短视频和长视频脚本创作",
        "category": "content",
        "recommended": True,
        "recommendedReason": "视频内容爆发增长，需求量大",
        "tags": ["视频", "脚本", "创意", "短视频"],
        "capabilities": ["黄金3秒开头设计", "情节结构编排", "钩子设置技巧", "评论区互动引导"],
        "scenarios": ["抖音/快手短视频", "B站长视频", "视频号内容", "直播脚本策划"],
        "dirName": "003.video-script-writer"
    },
    {
        "id": "pr-writer",
        "name": "公关稿件撰写",
        "description": "企业公关新闻稿和声明撰写",
        "category": "content",
        "recommended": False,
        "tags": ["公关", "新闻", "品牌", "写作"],
        "capabilities": ["新闻价值挖掘", "媒体角度包装", "危机公关话术", "品牌故事讲述"],
        "scenarios": ["产品发布新闻稿", "企业声明公告", "媒体通稿撰写", "危机公关处理"],
        "dirName": "004.pr-writer"
    },
    {
        "id": "knowledge-expert",
        "name": "知识科普专家",
        "description": "复杂知识通俗化解读",
        "category": "content",
        "recommended": False,
        "tags": ["科普", "教育", "知识", "教程"],
        "capabilities": ["专业概念降维", "类比解释能力", "图文结合表达", "互动问答设计"],
        "scenarios": ["技术科普文章", "行业知识解读", "产品使用教程", "FAQ知识库建设"],
        "dirName": "005.knowledge-expert"
    },
    {
        "id": "title-optimizer",
        "name": "标题优化大师",
        "description": "爆款标题创作和优化",
        "category": "content",
        "recommended": True,
        "recommendedReason": "标题决定打开率，实用价值高",
        "tags": ["标题", "优化", "转化", "创意"],
        "capabilities": ["标题党技巧运用", "好奇心激发", "数字化表达", "情绪词运用"],
        "scenarios": ["公众号标题优化", "短视频标题设计", "商品标题优化", "邮件主题优化"],
        "dirName": "006.title-optimizer"
    },

    # 二、市场营销类 (5个)
    {
        "id": "product-analyst",
        "name": "选品分析师",
        "description": "电商选品和市场趋势分析",
        "category": "marketing",
        "recommended": True,
        "recommendedReason": "电商从业核心需求",
        "tags": ["选品", "分析", "电商", "市场"],
        "capabilities": ["市场数据分析", "竞品监控追踪", "爆款特征识别", "供应链信息整合"],
        "scenarios": ["跨境电商选品", "新品开发建议", "市场机会发现", "竞品策略分析"],
        "dirName": "007.product-analyst"
    },
    {
        "id": "seo-expert",
        "name": "SEO优化专家",
        "description": "搜索引擎优化和流量获取",
        "category": "marketing",
        "recommended": True,
        "recommendedReason": "SEO是长期流量获取的关键",
        "tags": ["SEO", "流量", "优化", "搜索"],
        "capabilities": ["关键词研究", "内容优化建议", "技术SEO诊断", "排名监控追踪"],
        "scenarios": ["网站SEO优化", "商品排名提升", "内容流量增长", "竞争对手分析"],
        "dirName": "008.seo-expert"
    },
    {
        "id": "ad-optimizer",
        "name": "广告投放优化师",
        "description": "广告投放效果优化和ROI提升",
        "category": "marketing",
        "recommended": False,
        "tags": ["广告", "投放", "优化", "ROI"],
        "capabilities": ["广告创意分析", "受众定位优化", "出价策略建议", "转化漏斗分析"],
        "scenarios": ["信息流广告优化", "搜索广告投放", "社交广告测试", "ROI效果分析"],
        "dirName": "009.ad-optimizer"
    },
    {
        "id": "growth-hacker",
        "name": "用户增长专家",
        "description": "用户获取和增长策略制定",
        "category": "marketing",
        "recommended": False,
        "tags": ["增长", "裂变", "运营", "策略"],
        "capabilities": ["增长漏斗设计", "A/B测试策划", "病毒传播机制", "留存策略优化"],
        "scenarios": ["用户增长策略", "裂变营销策划", "活动效果优化", "用户生命周期管理"],
        "dirName": "010.growth-hacker"
    },
    {
        "id": "brand-planner",
        "name": "品牌策划师",
        "description": "品牌定位和视觉策划",
        "category": "marketing",
        "recommended": False,
        "tags": ["品牌", "策划", "设计", "营销"],
        "capabilities": ["品牌故事讲述", "视觉风格建议", "品牌调性把控", "跨界营销策划"],
        "scenarios": ["品牌定位策略", "品牌视觉设计", "品牌营销活动", "品牌升级改造"],
        "dirName": "011.brand-planner"
    },

    # 三、数据分析类 (4个)
    {
        "id": "data-analyst",
        "name": "数据分析师",
        "description": "业务数据分析和洞察挖掘",
        "category": "data",
        "recommended": True,
        "recommendedReason": "数据驱动决策的基础能力",
        "tags": ["数据", "分析", "可视化", "洞察"],
        "capabilities": ["数据清洗处理", "可视化图表设计", "趋势分析判断", "业务建议输出"],
        "scenarios": ["销售数据分析", "用户行为分析", "业务趋势预测", "数据报告生成"],
        "dirName": "002.data-analyst"  # 注意：这个目录名已存在，应该是 012
    },
    {
        "id": "competitor-monitor",
        "name": "竞品监控专员",
        "description": "竞品动态监控和策略分析",
        "category": "data",
        "recommended": False,
        "tags": ["竞品", "监控", "分析", "市场"],
        "capabilities": ["多平台监控", "价格变动追踪", "功能更新分析", "营销活动记录"],
        "scenarios": ["竞品价格监控", "功能迭代追踪", "营销活动分析", "市场动态报告"],
        "dirName": "013.competitor-monitor"
    },
    {
        "id": "financial-analyst",
        "name": "财务报表分析师",
        "description": "财务数据分析和报表解读",
        "category": "data",
        "recommended": False,
        "tags": ["财务", "分析", "报表", "预算"],
        "capabilities": ["财务指标解读", "趋势分析判断", "成本优化建议", "预算规划协助"],
        "scenarios": ["月度财务分析", "成本控制建议", "预算规划协助", "财务健康诊断"],
        "dirName": "014.financial-analyst"
    },
    {
        "id": "user-insight-analyst",
        "name": "用户洞察分析师",
        "description": "用户行为分析和需求挖掘",
        "category": "data",
        "recommended": False,
        "tags": ["用户", "洞察", "行为", "体验"],
        "capabilities": ["用户分群分析", "行为路径追踪", "需求痛点挖掘", "用户体验诊断"],
        "scenarios": ["用户画像构建", "使用行为分析", "需求优先级排序", "体验问题诊断"],
        "dirName": "015.user-insight-analyst"
    },

    # 四、项目管理类 (4个)
    {
        "id": "pm-assistant",
        "name": "项目经理助理",
        "description": "项目进度跟踪和团队协调",
        "category": "project",
        "recommended": True,
        "recommendedReason": "项目管理通用需求",
        "tags": ["项目", "管理", "协调", "进度"],
        "capabilities": ["进度监控提醒", "风险识别预警", "资源协调调度", "会议纪要整理"],
        "scenarios": ["项目进度跟踪", "里程碑管理", "团队协作协调", "项目报告生成"],
        "dirName": "016.pm-assistant"
    },
    {
        "id": "agile-coach",
        "name": "敏捷开发教练",
        "description": "敏捷流程指导和Scrum管理",
        "category": "project",
        "recommended": False,
        "tags": ["敏捷", "Scrum", "流程", "效率"],
        "capabilities": ["敏捷流程设计", "Sprint规划协助", "回顾会议引导", "团队效率提升"],
        "scenarios": ["敏捷流程导入", "迭代规划协助", "回顾会议主持", "团队协作优化"],
        "dirName": "017.agile-coach"
    },
    {
        "id": "risk-manager",
        "name": "风险管理专员",
        "description": "项目风险识别和应对策略",
        "category": "project",
        "recommended": False,
        "tags": ["风险", "管理", "评估", "应对"],
        "capabilities": ["风险识别评估", "应对方案制定", "风险监控预警", "危机处理协助"],
        "scenarios": ["项目风险评估", "风险应对规划", "危机事件处理", "风险报告生成"],
        "dirName": "018.risk-manager"
    },
    {
        "id": "okr-manager",
        "name": "OKR目标管理师",
        "description": "OKR制定和跟踪管理",
        "category": "project",
        "recommended": False,
        "tags": ["OKR", "目标", "管理", "复盘"],
        "capabilities": ["目标拆解协助", "关键结果定义", "进度跟踪提醒", "复盘总结引导"],
        "scenarios": ["季度OKR制定", "目标跟踪管理", "周进度回顾", "复盘总结报告"],
        "dirName": "019.okr-manager"
    },

    # 五、客户服务类 (3个)
    {
        "id": "customer-service",
        "name": "智能客服专家",
        "description": "客户问题解答和服务优化",
        "category": "service",
        "recommended": True,
        "recommendedReason": "客服场景高频需求",
        "tags": ["客服", "服务", "话术", "质量"],
        "capabilities": ["常见问题解答", "话术标准制定", "客诉处理协助", "服务质检评估"],
        "scenarios": ["在线客服支持", "投诉处理协助", "服务话术优化", "客户满意度提升"],
        "dirName": "020.customer-service"
    },
    {
        "id": "sales-coach",
        "name": "销售谈判教练",
        "description": "销售技巧和谈判策略指导",
        "category": "service",
        "recommended": False,
        "tags": ["销售", "谈判", "技巧", "成单"],
        "capabilities": ["客户需求分析", "异议处理话术", "谈判策略建议", "成单技巧传授"],
        "scenarios": ["销售话术优化", "谈判策略制定", "客户异议处理", "成单率提升"],
        "dirName": "021.sales-coach"
    },
    {
        "id": "customer-success",
        "name": "客户成功经理",
        "description": "客户续费和满意度管理",
        "category": "service",
        "recommended": False,
        "tags": ["续费", "成功", "流失", "增购"],
        "capabilities": ["续费策略制定", "流失风险预警", "增购机会挖掘", "成功案例分析"],
        "scenarios": ["续费策略规划", "流失预防处理", "增购机会识别", "客户成功案例"],
        "dirName": "022.customer-success"
    },

    # 六、技术开发类 (4个)
    {
        "id": "code-reviewer",
        "name": "代码审查专家",
        "description": "代码质量审查和最佳实践建议",
        "category": "development",
        "recommended": True,
        "recommendedReason": "代码质量保障必备",
        "tags": ["代码", "审查", "质量", "安全"],
        "capabilities": ["代码规范检查", "性能问题识别", "安全漏洞扫描", "重构建议提供"],
        "scenarios": ["代码审查执行", "技术债务管理", "性能优化建议", "安全问题排查"],
        "dirName": "023.code-reviewer"
    },
    {
        "id": "tech-writer",
        "name": "技术文档工程师",
        "description": "技术文档编写和维护",
        "category": "development",
        "recommended": False,
        "tags": ["文档", "技术", "API", "手册"],
        "capabilities": ["API文档编写", "用户手册制作", "故障排查指南", "技术方案整理"],
        "scenarios": ["API文档维护", "用户手册编写", "故障排查指南", "技术方案文档"],
        "dirName": "024.tech-writer"
    },
    {
        "id": "test-designer",
        "name": "测试用例设计师",
        "description": "测试用例设计和质量管理",
        "category": "development",
        "recommended": False,
        "tags": ["测试", "用例", "质量", "Bug"],
        "capabilities": ["功能测试设计", "边界条件分析", "测试用例编写", "Bug报告整理"],
        "scenarios": ["功能测试用例", "边界测试设计", "自动化测试规划", "Bug报告模板"],
        "dirName": "025.test-designer"
    },
    {
        "id": "architect-consultant",
        "name": "架构设计顾问",
        "description": "技术架构设计和优化建议",
        "category": "development",
        "recommended": False,
        "tags": ["架构", "设计", "性能", "扩展"],
        "capabilities": ["系统架构设计", "技术选型建议", "性能优化方案", "扩展性评估"],
        "scenarios": ["系统架构设计", "技术选型建议", "性能瓶颈分析", "扩展性评估"],
        "dirName": "026.architect-consultant"
    },

    # 七、行政人资类 (4个)
    {
        "id": "recruiter",
        "name": "招聘面试官",
        "description": "招聘流程和面试技巧指导",
        "category": "admin",
        "recommended": False,
        "tags": ["招聘", "面试", "JD", "简历"],
        "capabilities": ["岗位需求分析", "简历筛选标准", "面试问题设计", "候选人评估协助"],
        "scenarios": ["JD撰写优化", "简历筛选协助", "面试问题设计", "候选人评估报告"],
        "dirName": "027.recruiter"
    },
    {
        "id": "training-designer",
        "name": "培训课程设计师",
        "description": "培训课程设计和学习路径规划",
        "category": "admin",
        "recommended": False,
        "tags": ["培训", "课程", "学习", "路径"],
        "capabilities": ["课程体系设计", "学习路径规划", "培训材料制作", "效果评估设计"],
        "scenarios": ["新员工培训设计", "技能提升课程", "学习路径规划", "培训效果评估"],
        "dirName": "028.training-designer"
    },
    {
        "id": "compensation-specialist",
        "name": "薪酬福利专员",
        "description": "薪酬设计和福利方案优化",
        "category": "admin",
        "recommended": False,
        "tags": ["薪酬", "福利", "激励", "调研"],
        "capabilities": ["薪酬市场调研", "薪酬结构设计", "福利方案规划", "激励机制设计"],
        "scenarios": ["薪酬水平调研", "薪酬结构调整", "福利方案优化", "激励机制设计"],
        "dirName": "029.compensation-specialist"
    },
    {
        "id": "efficiency-expert",
        "name": "办公效率提升专家",
        "description": "办公流程优化和效率提升",
        "category": "admin",
        "recommended": True,
        "recommendedReason": "办公效率提升是通用需求",
        "tags": ["效率", "流程", "工具", "自动化"],
        "capabilities": ["流程梳理优化", "工具推荐配置", "自动化方案设计", "效率评估改进"],
        "scenarios": ["流程优化建议", "工具选型推荐", "自动化流程设计", "效率评估报告"],
        "dirName": "030.efficiency-expert"
    },
]

# 分类元数据
CATEGORY_METADATA = {
    'content': {'icon': '✍️', 'name': '内容创作', 'description': '文案、脚本、科普等内容相关岗位'},
    'marketing': {'icon': '📣', 'name': '市场营销', 'description': '选品、SEO、广告投放等营销岗位'},
    'data': {'icon': '📊', 'name': '数据分析', 'description': '数据、竞品、财务等分析岗位'},
    'project': {'icon': '📋', 'name': '项目管理', 'description': '项目经理、敏捷教练、OKR管理等'},
    'service': {'icon': '💬', 'name': '客户服务', 'description': '客服、销售、客户成功等'},
    'development': {'icon': '💻', 'name': '技术开发', 'description': '代码审查、文档、测试、架构等'},
    'admin': {'icon': '🏢', 'name': '行政人资', 'description': '招聘、培训、薪酬、办公效率等'},
}

def generate_manifest(workspaces_root: str, output_path: str):
    """生成 agent-workspaces-manifest.json"""

    workspaces = []
    workspaces_dir = Path(workspaces_root)

    # 检查目录是否存在
    if not workspaces_dir.exists():
        print(f"警告: Workspace 目录不存在: {workspaces_root}")
        print("将使用相对路径生成 manifest")

    for ws_data in WORKSPACES_DATA:
        category_meta = CATEGORY_METADATA.get(ws_data['category'], {})

        # 构建路径 - 使用 id 而不是 dirName
        if workspaces_dir.exists():
            workspace_path = workspaces_dir / ws_data['id']
        else:
            # 开发环境下的相对路径
            workspace_path = Path(f"docs/Agents预设系统/Workspaces/{ws_data['id']}")

        workspace = {
            'id': ws_data['id'],
            'name': ws_data['name'],
            'description': ws_data['description'],
            'icon': category_meta.get('icon', '📦'),
            'category': ws_data['category'],
            'recommended': ws_data.get('recommended', False),
            'recommendedReason': ws_data.get('recommendedReason'),
            'tags': ws_data['tags'],
            'capabilities': ws_data['capabilities'],
            'scenarios': ws_data['scenarios'],
            'path': str(workspace_path.absolute()) if workspaces_dir.exists() else str(workspace_path),
        }

        workspaces.append(workspace)

    # 生成 manifest
    manifest = {
        'version': '1.0.0',
        'updatedAt': datetime.now().isoformat(),
        'workspaces': workspaces,
    }

    # 写入文件
    output_file = Path(output_path)
    output_file.parent.mkdir(parents=True, exist_ok=True)

    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(manifest, f, ensure_ascii=False, indent=2)

    print(f"✅ 已生成 manifest: {output_path}")
    print(f"📊 共包含 {len(workspaces)} 个 Workspace")
    print(f"\n分类统计:")
    for cat_id, cat_meta in CATEGORY_METADATA.items():
        count = sum(1 for ws in workspaces if ws['category'] == cat_id)
        if count > 0:
            print(f"  {cat_meta['icon']} {cat_meta['name']}: {count}个")

    # 统计推荐岗位
    recommended_count = sum(1 for ws in workspaces if ws['recommended'])
    print(f"\n⭐ 推荐岗位: {recommended_count}个")

if __name__ == '__main__':
    # 获取项目根目录
    script_dir = Path(__file__).parent
    project_root = script_dir.parent

    workspaces_root = project_root / 'docs' / 'Agents预设系统' / 'Workspaces'
    output_path = project_root / 'src-tauri' / 'resources' / 'agent-workspaces-manifest.json'

    generate_manifest(str(workspaces_root), str(output_path))
