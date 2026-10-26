// ============================================================
// BBKEmu — Landing Page Scripts
// i18n + Carousel Gallery + Animations
// ============================================================

(function () {
  'use strict';

  // ================================================================
  // GAME DATA — All 152 BBK games (from GAME-COMPATIBILITY.md)
  // ================================================================
  var GAMES = [
    // RPG / Adventure (84)
    { cat: 'rpg', zh: '一中传奇', en: 'Yi Zhong Chuanqi', img: 'docs/images/一中传奇.png' },
    { cat: 'rpg', zh: '一中传奇2', en: 'Yi Zhong Chuanqi 2', img: 'docs/images/一中传奇2.png' },
    { cat: 'rpg', zh: '七剑', en: 'Seven Swords', img: 'docs/images/七剑.png' },
    { cat: 'rpg', zh: '三国霸业', en: 'Three Kingdoms', img: 'docs/images/三国霸业.png' },
    { cat: 'rpg', zh: '仙三外传', en: 'Palace III Side Story', img: 'docs/images/仙三外传.png' },
    { cat: 'rpg', zh: '仙剑三', en: 'Palace III', img: 'docs/images/仙剑三.png' },
    { cat: 'rpg', zh: '仙剑奇侠传二之虎啸飞剑', en: 'Palace II: Tiger Roar', img: 'docs/images/仙剑奇侠传二之虎啸飞剑.png' },
    { cat: 'rpg', zh: '仙剑奇侠传四回梦游仙', en: 'Palace IV: Dream Journey', img: 'docs/images/仙剑奇侠传四回梦游仙.png' },
    { cat: 'rpg', zh: '仙界传说', en: 'Immortal Legend', img: 'docs/images/仙界传说.png' },
    { cat: 'rpg', zh: '伏魔记', en: 'Demon Subduer', img: 'docs/images/伏魔记.png' },
    { cat: 'rpg', zh: '伏魔记(有声版)', en: 'Demon Subduer (Audio)', img: 'docs/images/伏魔记(有声版).png' },
    { cat: 'rpg', zh: '伏魔记 加秘籍', en: 'Demon Subduer + Cheats', img: 'docs/images/伏魔记 加秘籍.png' },
    { cat: 'rpg', zh: '伏魔记-新护神记', en: 'Demon Subduer: New Guardian', img: 'docs/images/伏魔记-新护神记.png' },
    { cat: 'rpg', zh: '伏魔记-清风传', en: 'Demon Subduer: Breeze', img: 'docs/images/伏魔记-清风传.png' },
    { cat: 'rpg', zh: '伏魔记-游戏王', en: 'Demon Subduer: Game King', img: 'docs/images/伏魔记-游戏王.png' },
    { cat: 'rpg', zh: '伏魔记-王柱人传奇', en: 'Demon Subduer: Wang Zhu', img: 'docs/images/伏魔记-王柱人传奇.png' },
    { cat: 'rpg', zh: '伏魔记-伏魔记外传', en: 'Demon Subduer: Side Story', img: 'docs/images/伏魔记-伏魔记外传.png' },
    { cat: 'rpg', zh: '伏魔记-魔道传奇', en: 'Demon Subduer: Dark Path', img: 'docs/images/伏魔记-魔道传奇.png' },
    { cat: 'rpg', zh: '伏魔记之圆梦间奏曲v0.1', en: 'Demon Subduer: Dream v0.1', img: 'docs/images/伏魔记之圆梦间奏曲v0.1.png' },
    { cat: 'rpg', zh: '伏魔记圆梦前奏曲(公测版)', en: 'Demon Subduer: Prelude', img: 'docs/images/伏魔记圆梦前奏曲(公测版).png' },
    { cat: 'rpg', zh: '伏魔记怀旧终曲v1.0(原版精修)', en: 'Demon Subduer: Finale v1.0', img: 'docs/images/伏魔记怀旧终曲v1.0(原版精修).png' },
    { cat: 'rpg', zh: '伏魔迷宫', en: 'Demon Maze', img: 'docs/images/伏魔迷宫.png' },
    { cat: 'rpg', zh: '侠客行', en: 'Swordsman Journey', img: 'docs/images/侠客行.png' },
    { cat: 'rpg', zh: '侠客行4988(终曲版)', en: 'Swordsman 4988 Finale', img: 'docs/images/侠客行4988(终曲版).png' },
    { cat: 'rpg', zh: '剑缘-第一部', en: 'Sword Fate Part 1', img: 'docs/images/剑缘-第一部.png' },
    { cat: 'rpg', zh: '十字之门', en: 'Cross Gate', img: 'docs/images/十字之门.png' },
    { cat: 'rpg', zh: '同福奇缘', en: 'Tongfu Fortune', img: 'docs/images/同福奇缘.png' },
    { cat: 'rpg', zh: '地牢围攻', en: 'Dungeon Siege', img: 'docs/images/地牢围攻.png' },
    { cat: 'rpg', zh: '基督山传奇', en: 'Monte Cristo', img: 'docs/images/基督山传奇.png' },
    { cat: 'rpg', zh: '天之骄子', en: 'Pride of Heaven', img: 'docs/images/天之骄子.png' },
    { cat: 'rpg', zh: '天之骄子终曲版', en: 'Pride of Heaven Finale', img: 'docs/images/天之骄子终曲版.png' },
    { cat: 'rpg', zh: '妖 传说', en: 'Demon Legend', img: 'docs/images/妖 传说.png' },
    { cat: 'rpg', zh: '妖·传说终曲版', en: 'Demon Legend Finale', img: 'docs/images/妖·传说终曲版.png' },
    { cat: 'rpg', zh: '封魔录', en: 'Demon Seal', img: 'docs/images/封魔录.png' },
    { cat: 'rpg', zh: '将门风云', en: 'General Gate', img: 'docs/images/将门风云.png' },
    { cat: 'rpg', zh: '少年行', en: 'Youth Journey', img: 'docs/images/少年行.png' },
    { cat: 'rpg', zh: '屠魔', en: 'Demon Slayer', img: 'docs/images/屠魔.png' },
    { cat: 'rpg', zh: '异世大陆', en: 'Other World', img: 'docs/images/异世大陆.png' },
    { cat: 'rpg', zh: '异时空游记', en: 'Time Travel', img: 'docs/images/异时空游记.png' },
    { cat: 'rpg', zh: '异时空游记2-纵横之旅', en: 'Time Travel 2', img: 'docs/images/异时空游记2-纵横之旅.png' },
    { cat: 'rpg', zh: '志在青云', en: 'Ambition', img: 'docs/images/志在青云.png' },
    { cat: 'rpg', zh: '恶龙传说', en: 'Evil Dragon', img: 'docs/images/恶龙传说.png' },
    { cat: 'rpg', zh: '我的世界', en: 'My World', img: 'docs/images/我的世界.png' },
    { cat: 'rpg', zh: '战国争霸', en: 'Warring States', img: 'docs/images/战国争霸.png' },
    { cat: 'rpg', zh: '新仙剑奇侠传', en: 'New Palace', img: 'docs/images/新仙剑奇侠传.png' },
    { cat: 'rpg', zh: '新仙剑奇侠传终曲版', en: 'New Palace Finale', img: 'docs/images/新仙剑奇侠传终曲版.png' },
    { cat: 'rpg', zh: '新伏魔记', en: 'New Demon Subduer', img: 'docs/images/新伏魔记.png' },
    { cat: 'rpg', zh: '新伏魔S终曲版', en: 'New Demon S Finale', img: 'docs/images/新伏魔S终曲版.png' },
    { cat: 'rpg', zh: '末日传说', en: 'Doomsday Legend', img: 'docs/images/末日传说.png' },
    { cat: 'rpg', zh: '校园传奇', en: 'Campus Legend', img: 'docs/images/校园传奇.png' },
    { cat: 'rpg', zh: '梦幻校园', en: 'Dream Campus', img: 'docs/images/梦幻校园.png' },
    { cat: 'rpg', zh: '梦幻西游', en: 'Dream Journey West', img: 'docs/images/梦幻西游.png' },
    { cat: 'rpg', zh: '武林新传', en: 'Martial World', img: 'docs/images/武林新传.png' },
    { cat: 'rpg', zh: '洛特传奇', en: 'Lotte Legend', img: 'docs/images/洛特传奇.png' },
    { cat: 'rpg', zh: '海盗船', en: 'Pirate Ship', img: 'docs/images/海盗船.png' },
    { cat: 'rpg', zh: '混战三国', en: 'Three Kingdoms Brawl', img: 'docs/images/混战三国.png' },
    { cat: 'rpg', zh: '热血传奇', en: 'Hot Blood Legend', img: 'docs/images/热血传奇.png' },
    { cat: 'rpg', zh: '牛妞历险记', en: 'Niu Niu Adventure', img: 'docs/images/牛妞历险记.png' },
    { cat: 'rpg', zh: '王氏传', en: 'Wang Story', img: 'docs/images/王氏传.png' },
    { cat: 'rpg', zh: '生命女神之暗之诅咒', en: 'Goddess: Dark Curse', img: 'docs/images/生命女神之暗之诅咒.png' },
    { cat: 'rpg', zh: '诸神黄昏', en: 'Ragnarok', img: 'docs/images/诸神黄昏.png' },
    { cat: 'rpg', zh: '豪斯', en: 'House', img: 'docs/images/豪斯.png' },
    { cat: 'rpg', zh: '金庸群侠传', en: 'Jin Yong Heroes', img: 'docs/images/金庸群侠传.png' },
    { cat: 'rpg', zh: '金庸群侠传终曲版', en: 'Jin Yong Heroes Finale', img: 'docs/images/金庸群侠传终曲版.png' },
    { cat: 'rpg', zh: '金庸群侠黑暗时代终曲版', en: 'Jin Yong Dark Age', img: 'docs/images/金庸群侠黑暗时代终曲版.png' },
    { cat: 'rpg', zh: '阶梯小子', en: 'Stair Boy', img: 'docs/images/阶梯小子.png' },
    { cat: 'rpg', zh: '英雄剑', en: 'Hero Sword', img: 'docs/images/英雄剑.png' },
    { cat: 'rpg', zh: '英雄剑1', en: 'Hero Sword 1', img: 'docs/images/英雄剑1.png' },
    { cat: 'rpg', zh: '英雄剑2', en: 'Hero Sword 2', img: 'docs/images/英雄剑2.png' },
    { cat: 'rpg', zh: '英雄坛', en: 'Hero Altar', img: 'docs/images/英雄坛.png' },
    { cat: 'rpg', zh: '英雄坛说', en: 'Hero Altar Story', img: 'docs/images/英雄坛说.png' },
    { cat: 'rpg', zh: '英雄坛说终曲版', en: 'Hero Altar Finale', img: 'docs/images/英雄坛说终曲版.png' },
    { cat: 'rpg', zh: '英雄战士', en: 'Hero Warrior', img: 'docs/images/英雄战士.png' },
    { cat: 'rpg', zh: '落世沉浮', en: 'World Ups Downs', img: 'docs/images/落世沉浮.png' },
    { cat: 'rpg', zh: '蓝色天际', en: 'Blue Horizon', img: 'docs/images/蓝色天际.png' },
    { cat: 'rpg', zh: '魔塔', en: 'Magic Tower', img: 'docs/images/魔塔.png' },
    { cat: 'rpg', zh: '魔塔BT版', en: 'Magic Tower BT', img: 'docs/images/魔塔BT版.png' },
    { cat: 'rpg', zh: '魔塔超级版', en: 'Magic Tower Super', img: 'docs/images/魔塔超级版.png' },
    { cat: 'rpg', zh: '魔法学院', en: 'Magic Academy', img: 'docs/images/魔法学院.png' },
    { cat: 'rpg', zh: '黑暗之心', en: 'Heart of Darkness', img: 'docs/images/黑暗之心.png' },
    { cat: 'rpg', zh: '白中传奇', en: 'White Legend', img: 'docs/images/白中传奇.png' },
    { cat: 'rpg', zh: '紫璇刀', en: 'Purple Blade', img: 'docs/images/紫璇刀.png' },
    { cat: 'rpg', zh: '纯蓝记', en: 'Pure Blue', img: 'docs/images/纯蓝记.png' },
    { cat: 'rpg', zh: '老观寺传奇 加秘籍', en: 'Temple Legend + Cheats', img: 'docs/images/老观寺传奇 加秘籍.png' },
    { cat: 'rpg', zh: '老观寺传奇终曲版', en: 'Temple Legend Finale', img: 'docs/images/老观寺传奇终曲版.png' },

    // Puzzle / Strategy (21)
    { cat: 'puzzle', zh: 'Eros方块', en: 'Eros Blocks', img: 'docs/images/Eros方块.png' },
    { cat: 'puzzle', zh: '中国象棋', en: 'Chinese Chess', img: 'docs/images/中国象棋.png' },
    { cat: 'puzzle', zh: '二十一点', en: 'Blackjack', img: 'docs/images/二十一点.png' },
    { cat: 'puzzle', zh: '二十四点', en: '24 Points', img: 'docs/images/二十四点.png' },
    { cat: 'puzzle', zh: '五子棋', en: 'Gomoku', img: 'docs/images/五子棋.png' },
    { cat: 'puzzle', zh: '升级', en: 'Sheng Ji', img: 'docs/images/升级.png' },
    { cat: 'puzzle', zh: '华容道', en: 'Huarong Road', img: 'docs/images/华容道.png' },
    { cat: 'puzzle', zh: '对对碰', en: 'Match Match', img: 'docs/images/对对碰.png' },
    { cat: 'puzzle', zh: '平面魔方', en: 'Flat Cube', img: 'docs/images/平面魔方.png' },
    { cat: 'puzzle', zh: '扫雷', en: 'Minesweeper', img: 'docs/images/扫雷.png' },
    { cat: 'puzzle', zh: '拱猪', en: 'Pig Card Game', img: 'docs/images/拱猪.png' },
    { cat: 'puzzle', zh: '接龙', en: 'Solitaire', img: 'docs/images/接龙.png' },
    { cat: 'puzzle', zh: '搬运工', en: 'Sokoban', img: 'docs/images/搬运工.png' },
    { cat: 'puzzle', zh: '比大小', en: 'Big or Small', img: 'docs/images/比大小.png' },
    { cat: 'puzzle', zh: '智多星', en: 'Mastermind', img: 'docs/images/智多星.png' },
    { cat: 'puzzle', zh: '黑白子', en: 'Reversi', img: 'docs/images/黑白子.png' },
    { cat: 'puzzle', zh: '碰碰车', en: 'Bumper Cars', img: 'docs/images/碰碰车.png' },
    { cat: 'puzzle', zh: '蜘蛛侠三', en: 'Spider-Man 3', img: 'docs/images/蜘蛛侠三.png' },
    { cat: 'puzzle', zh: '螃蟹回家', en: 'Crab Home', img: 'docs/images/螃蟹回家.png' },
    { cat: 'puzzle', zh: '贪食蛇', en: 'Snake', img: 'docs/images/贪食蛇.png' },
    { cat: 'puzzle', zh: '幸运花', en: 'Lucky Flower', img: 'docs/images/幸运花.png' },

    // Action / Arcade (43)
    { cat: 'action', zh: 'DIYtheGAME', en: 'DIY the GAME', img: 'docs/images/DIYtheGAME.png' },
    { cat: 'action', zh: '乒乓球', en: 'Ping Pong', img: 'docs/images/乒乓球.png' },
    { cat: 'action', zh: '丰收', en: 'Harvest', img: 'docs/images/丰收.png' },
    { cat: 'action', zh: '体闲麻将', en: 'Casual Mahjong', img: 'docs/images/体闲麻将.png' },
    { cat: 'action', zh: '公路快车', en: 'Highway Express', img: 'docs/images/公路快车.png' },
    { cat: 'action', zh: '冒险岛', en: 'Adventure Island', img: 'docs/images/冒险岛.png' },
    { cat: 'action', zh: '坦克大战', en: 'Tank Battle', img: 'docs/images/坦克大战.png' },
    { cat: 'action', zh: '大乱斗之火影忍者', en: 'Naruto Brawl', img: 'docs/images/大乱斗之火影忍者.png' },
    { cat: 'action', zh: '大话三国', en: 'Three Kingdoms Talk', img: 'docs/images/大话三国.png' },
    { cat: 'action', zh: '宠物精灵', en: 'Pet Monster', img: 'docs/images/宠物精灵.png' },
    { cat: 'action', zh: '电子宠物', en: 'Digital Pet', img: 'docs/images/电子宠物.png' },
    { cat: 'action', zh: '娱乐无极限之无影奸细', en: 'Shadow Spy', img: 'docs/images/娱乐无极限之无影奸细.png' },
    { cat: 'action', zh: '投篮游戏', en: 'Basketball', img: 'docs/images/投篮游戏.png' },
    { cat: 'action', zh: '抗日小兵', en: 'Anti-Japan Soldier', img: 'docs/images/抗日小兵.png' },
    { cat: 'action', zh: '挖金子', en: 'Gold Digger', img: 'docs/images/挖金子.png' },
    { cat: 'action', zh: '泡泡侠', en: 'Bubble Hero', img: 'docs/images/泡泡侠.png' },
    { cat: 'action', zh: '泡泡侠 加速版', en: 'Bubble Hero Turbo', img: 'docs/images/泡泡侠 加速版.png' },
    { cat: 'action', zh: '洛克人', en: 'Rockman', img: 'docs/images/洛克人.png' },
    { cat: 'action', zh: '滑雪', en: 'Skiing', img: 'docs/images/滑雪.png' },
    { cat: 'action', zh: '潜艇大战', en: 'Submarine War', img: 'docs/images/潜艇大战.png' },
    { cat: 'action', zh: '炸弹小子', en: 'Bomberman', img: 'docs/images/炸弹小子.png' },
    { cat: 'action', zh: '烈中轶事', en: 'Fire Story', img: 'docs/images/烈中轶事.png' },
    { cat: 'action', zh: '猪小弟', en: 'Piggy', img: 'docs/images/猪小弟.png' },
    { cat: 'action', zh: '猫狗大战', en: 'Cat vs Dog', img: 'docs/images/猫狗大战.png' },
    { cat: 'action', zh: '赛马', en: 'Horse Racing', img: 'docs/images/赛马.png' },
    { cat: 'action', zh: '赤壁之战 乱世枭雄', en: 'Battle of Red Cliffs', img: 'docs/images/赤壁之战 乱世枭雄.png' },
    { cat: 'action', zh: '赤壁之战乱世枭雄终曲版', en: 'Red Cliffs Finale', img: 'docs/images/赤壁之战乱世枭雄终曲版.png' },
    { cat: 'action', zh: '跟花', en: 'Follow Flower', img: 'docs/images/跟花.png' },
    { cat: 'action', zh: '跳蛋', en: 'Jumping Egg', img: 'docs/images/跳蛋.png' },
    { cat: 'action', zh: '过关斩将', en: 'Stage Clear', img: 'docs/images/过关斩将.png' },
    { cat: 'action', zh: '过关斩将4988(终曲版)', en: 'Stage Clear 4988', img: 'docs/images/过关斩将4988(终曲版).png' },
    { cat: 'action', zh: '过关斩将4988改主角', en: 'Stage Clear 4988 Mod', img: 'docs/images/过关斩将4988改主角.png' },
    { cat: 'action', zh: '迷宫游戏', en: 'Maze Game', img: 'docs/images/迷宫游戏.png' },
    { cat: 'action', zh: '遗忘传说', en: 'Forgotten Legend', img: 'docs/images/遗忘传说.png' },
    { cat: 'action', zh: '遗忘传说终曲版', en: 'Forgotten Legend Finale', img: 'docs/images/遗忘传说终曲版.png' },
    { cat: 'action', zh: '释厄传', en: 'Exorcism', img: 'docs/images/释厄传.png' },
    { cat: 'action', zh: '钓鱼', en: 'Fishing', img: 'docs/images/钓鱼.png' },
    { cat: 'action', zh: '钓鲨鱼', en: 'Shark Fishing', img: 'docs/images/钓鲨鱼.png' },
    { cat: 'action', zh: '问道', en: 'Ask the Way', img: 'docs/images/问道.png' },
    { cat: 'action', zh: '飞行特训', en: 'Flight Training', img: 'docs/images/飞行特训.png' },
    { cat: 'action', zh: '疯狂校园', en: 'Crazy Campus', img: 'docs/images/疯狂校园.png' },
    { cat: 'action', zh: '疯狂盗墓人', en: 'Tomb Raider', img: 'docs/images/疯狂盗墓人.png' },
    { cat: 'action', zh: '秘密潜入', en: 'Stealth Ops', img: 'docs/images/秘密潜入.png' },

    // Other (4)
    { cat: 'other', zh: '步步高网友俱乐部', en: 'BBK Fan Club', img: 'docs/images/步步高网友俱乐部.png' },
    { cat: 'other', zh: '新能源危机', en: 'Energy Crisis', img: 'docs/images/新能源危机.png' },
    { cat: 'other', zh: '最终幻想', en: 'Final Fantasy', img: 'docs/images/最终幻想.png' }
  ];

  var CATEGORIES = [
    { id: 'all', zh: '全部', en: 'All' },
    { id: 'rpg', zh: 'RPG / 冒险', en: 'RPG / Adventure' },
    { id: 'puzzle', zh: '益智 / 策略', en: 'Puzzle / Strategy' },
    { id: 'action', zh: '动作 / 街机', en: 'Action / Arcade' },
    { id: 'other', zh: '其他', en: 'Other' }
  ];

  // ================================================================
  // i18n — Translations
  // ================================================================
  var translations = {
    zh: {
      // nav
      'nav-features': '核心特性',
      'nav-games': '游戏库',
      'nav-arch': '技术架构',
      'nav-quickstart': '快速开始',
      'nav-play': '在线试玩',
      // hero
      'hero-subtitle': '让步步高电子词典游戏重获新生',
      'hero-desc': '用 Rust 编写的 BBK A 系列电子词典游戏模拟器，完整支持 152 款经典游戏',
      'hero-play': '在线试玩',
      'hero-download': '下载',
      'hero-github': '查看源码',
      'hero-scroll': '向下滚动探索',
      // about
      'about-title': '什么是步步高电子词典？',
      'about-p1': 'BBK（步步高）A 系列电子词典是 2000 年代风靡中国校园的掌上设备。除了词典功能，它还内置了 <strong>6502 CPU</strong>，运行着大量由爱好者编写的游戏——从 RPG、策略到动作冒险，种类丰富。',
      'about-p2': '这些游戏以 .gam 格式分发，运行在 159×96 像素的单色 LCD 屏幕上。如今通过 BBKEmu，你可以在现代设备上重温这些经典。',
      'about-chip': 'BBK 词典',
      'about-chip-sub': '6502 CPU',
      'about-game-sub': '152 款游戏',
      'about-emu-sub': 'Rust 模拟器',
      // stats
      'stat-games': '支持游戏',
      'stat-games-sub': '98% 通过测试',
      'stat-lcd': 'LCD 分辨率',
      'stat-lcd-sub': '单色液晶屏',
      'stat-platforms': '目标平台',
      'stat-platforms-sub': 'Windows / macOS / Linux / Android / Web',
      'stat-lines': 'Rust 代码行数',
      'stat-lines-sub': '零 C 依赖',
      // features
      'feat-title': '核心特性',
      'feat-subtitle': '从 6502 CPU 到 LCD 显示，全栈 Rust 实现',
      'feat-cpu-title': '6502 CPU 模拟',
      'feat-cpu-desc': '基于 mos6502 crate 的精确 6502 指令集模拟，完整支持 Bank-switched 内存映射，忠实还原步步高词典硬件。',
      'feat-lcd-title': 'LCD 显示模拟',
      'feat-lcd-desc': '159×96 像素单色 LCD 帧缓冲区，支持 LCD 残影效果和横竖屏切换，还原经典绿色液晶屏视觉体验。',
      'feat-key-title': '键盘输入映射',
      'feat-key-desc': '完整的 BBK 按键矩阵模拟，支持方向键、确认键、取消键等全部按键，带可配置的按键重复间隔。',
      'feat-audio-title': '音频系统',
      'feat-audio-desc': '可配置频率和时长的音频合成系统，还原步步高词典的蜂鸣器音效，支持游戏中的声音反馈。',
      'feat-cheat-title': '作弊码 & 存档',
      'feat-cheat-desc': '支持 GameShark 格式作弊码、Save/Load State 存档功能，以及 SRAM 闪存模拟，让游戏体验更加灵活。',
      'feat-retro-title': 'RetroArch 核心',
      'feat-retro-desc': '完整的 libretro 核心，支持 RetroPad 映射、核心选项（CPU/Timer 速率、LCD 方向）、Save State 等 RetroArch 生态功能。',
      // gallery
      'gallery-title': '游戏库',
      'gallery-subtitle': '152 款游戏，4 大分类，98% 通过测试',
      // architecture
      'arch-title': '技术架构',
      'arch-subtitle': '清晰的三层架构，平台无关的核心引擎',
      'arch-frontends': '前端',
      'arch-standalone-sub': 'Standalone 可执行文件<br>minifb 窗口',
      'arch-libretro-sub': 'libretro cdylib<br>RetroArch 核心',
      'arch-core': '核心引擎',
      'arch-core-sub': '平台无关的库',
      'arch-platforms': '目标平台',
      // play online
      'play-cta-badge': '无需安装 · 浏览器直接运行',
      'play-cta-title': '在线试玩 BBK 游戏',
      'play-cta-desc': '选择本机上的游戏文件与固件，即可在浏览器中直接运行。支持键盘、手柄与手机触屏，文件不会离开你的设备。',
      'play-cta-btn': '开始试玩',
      // code
      'code-title': '纯 Rust，零妥协',
      'code-subtitle': '从 6502 CPU 到 LCD 帧缓冲区，全部用 Rust 从零实现',
      // quickstart
      'qs-title': '快速开始',
      'qs-subtitle': '几行命令，即刻体验',
      'qs-standalone': 'Standalone',
      'qs-standalone-1': '下载最新版本',
      'qs-standalone-1-sub': '从 Releases 页面下载对应平台的二进制文件',
      'qs-standalone-2': '准备 ROM 文件',
      'qs-standalone-3': '或自动搜索 ROM',
      'qs-retro': 'RetroArch',
      'qs-retro-1': '下载 libretro 核心',
      'qs-retro-1-sub': '从 Releases 页面下载对应平台的核心文件',
      'qs-retro-2': '放置 ROM 文件',
      'qs-retro-2-sub': '复制到 system/BBKEmu/A4980/ 目录',
      'qs-retro-3': '加载核心并启动',
      'qs-build': '从源码编译',
      'qs-build-1': '克隆仓库',
      'qs-build-2': '编译 Standalone',
      'qs-build-3': '或编译 RetroArch 核心',
      // footer
      'footer-desc': '用 Rust 编写的 BBK 电子词典游戏模拟器',
      'footer-project': '项目',
      'footer-docs': '文档',
      'footer-browser': '浏览器',
      'footer-specs': '技术规格',
      'footer-arch': '架构',
      'footer-memmap': '内存映射',
      'footer-gamelist': '游戏兼容性',
      'footer-syscalls': '系统调用',
      'footer-gamformat': 'GAM 格式',
      'footer-copy': 'BSD 3-Clause License &copy; 2025 Aloys. Built with 🦀 Rust.'
    },
    en: {
      // nav
      'nav-features': 'Features',
      'nav-games': 'Games',
      'nav-arch': 'Architecture',
      'nav-quickstart': 'Quick Start',
      'nav-play': 'Play Online',
      // hero
      'hero-subtitle': 'Bring BBK electronic dictionary games back to life',
      'hero-desc': 'A BBK A-series electronic dictionary game emulator written in Rust, fully supporting 152 classic games',
      'hero-play': 'Play Online',
      'hero-download': 'Download',
      'hero-github': 'View Source',
      'hero-scroll': 'Scroll to explore',
      // about
      'about-title': 'What is BBK Electronic Dictionary?',
      'about-p1': 'The BBK A-series electronic dictionary was a hugely popular handheld device in Chinese schools during the 2000s. Beyond its dictionary functions, it featured a <strong>6502 CPU</strong> that ran a vast library of fan-made games — from RPGs and strategy to action adventures.',
      'about-p2': 'These games, distributed as .gam files and running on a 159×96 monochrome LCD screen, can now be relived on modern devices through BBKEmu.',
      'about-chip': 'BBK Dictionary',
      'about-chip-sub': '6502 CPU',
      'about-game-sub': '152 Games',
      'about-emu-sub': 'Rust Emulator',
      // stats
      'stat-games': 'Games Supported',
      'stat-games-sub': '98% passing tests',
      'stat-lcd': 'LCD Resolution',
      'stat-lcd-sub': 'Monochrome LCD',
      'stat-platforms': 'Platforms',
      'stat-platforms-sub': 'Windows / macOS / Linux / Android / Web',
      'stat-lines': 'Lines of Rust',
      'stat-lines-sub': 'Zero C dependencies',
      // features
      'feat-title': 'Core Features',
      'feat-subtitle': 'From 6502 CPU to LCD display — full-stack Rust implementation',
      'feat-cpu-title': '6502 CPU Emulation',
      'feat-cpu-desc': 'Accurate 6502 instruction set emulation based on the mos6502 crate, with full bank-switched memory mapping — faithfully reproducing BBK dictionary hardware.',
      'feat-lcd-title': 'LCD Display Emulation',
      'feat-lcd-desc': '159×96 monochrome LCD framebuffer with ghosting effect simulation and portrait/landscape toggle — recreating the classic green LCD visual experience.',
      'feat-key-title': 'Keyboard Input Mapping',
      'feat-key-desc': 'Complete BBK key matrix emulation with directional keys, confirm, cancel and all other buttons, featuring configurable key repeat intervals.',
      'feat-audio-title': 'Audio System',
      'feat-audio-desc': 'Configurable frequency and duration audio synthesis, reproducing the BBK dictionary buzzer sounds for in-game audio feedback.',
      'feat-cheat-title': 'Cheat Codes & Saves',
      'feat-cheat-desc': 'GameShark format cheat codes, Save/Load State functionality, and SRAM flash emulation for a more flexible gaming experience.',
      'feat-retro-title': 'RetroArch Core',
      'feat-retro-desc': 'Complete libretro core with RetroPad mapping, core options (CPU/Timer rate, LCD orientation), Save State and full RetroArch ecosystem support.',
      // gallery
      'gallery-title': 'Game Library',
      'gallery-subtitle': '152 games, 4 categories, 98% passing tests',
      // architecture
      'arch-title': 'Architecture',
      'arch-subtitle': 'Clean three-layer architecture with a platform-independent core engine',
      'arch-frontends': 'Frontends',
      'arch-standalone-sub': 'Standalone binary<br>minifb window',
      'arch-libretro-sub': 'libretro cdylib<br>RetroArch core',
      'arch-core': 'Core Engine',
      'arch-core-sub': 'Platform-independent library',
      'arch-platforms': 'Platforms',
      // play online
      'play-cta-badge': 'No install · runs in your browser',
      'play-cta-title': 'Play BBK Games Online',
      'play-cta-desc': 'Pick a game file and firmware from your device and run it right in the browser. Keyboard, gamepad and touch controls are supported, and files never leave your machine.',
      'play-cta-btn': 'Start Playing',
      // code
      'code-title': 'Pure Rust, Zero Compromise',
      'code-subtitle': 'From 6502 CPU to LCD framebuffer — everything built from scratch in Rust',
      // quickstart
      'qs-title': 'Quick Start',
      'qs-subtitle': 'A few commands to get started',
      'qs-standalone': 'Standalone',
      'qs-standalone-1': 'Download latest release',
      'qs-standalone-1-sub': 'Get the binary for your platform from the Releases page',
      'qs-standalone-2': 'Prepare ROM files',
      'qs-standalone-3': 'Or auto-search ROMs',
      'qs-retro': 'RetroArch',
      'qs-retro-1': 'Download libretro core',
      'qs-retro-1-sub': 'Get the core for your platform from the Releases page',
      'qs-retro-2': 'Place ROM files',
      'qs-retro-2-sub': 'Copy to system/BBKEmu/A4980/ directory',
      'qs-retro-3': 'Load core and start',
      'qs-build': 'Build from Source',
      'qs-build-1': 'Clone the repository',
      'qs-build-2': 'Build Standalone',
      'qs-build-3': 'Or build RetroArch core',
      // footer
      'footer-desc': 'A BBK electronic dictionary game emulator written in Rust',
      'footer-project': 'Project',
      'footer-docs': 'Docs',
      'footer-browser': 'Browser',
      'footer-specs': 'Specs',
      'footer-arch': 'Architecture',
      'footer-memmap': 'Memory Map',
      'footer-gamelist': 'Game Compatibility',
      'footer-syscalls': 'System Calls',
      'footer-gamformat': 'GAM Format',
      'footer-copy': 'BSD 3-Clause License &copy; 2025 Aloys. Built with 🦀 Rust.'
    }
  };

  var currentLang = localStorage.getItem('bbk-lang') || (navigator.language.startsWith('zh') ? 'zh' : 'en');

  // ================================================================
  // i18n — Apply translations
  // ================================================================
  function applyLang(lang) {
    currentLang = lang;
    localStorage.setItem('bbk-lang', lang);
    document.documentElement.lang = lang === 'zh' ? 'zh-CN' : 'en';

    var t = translations[lang];

    document.querySelectorAll('[data-i18n]').forEach(function (el) {
      var key = el.getAttribute('data-i18n');
      if (t[key] === undefined) return;
      if (el.tagName === 'TITLE' || el.tagName === 'META') return;
      el.innerHTML = t[key];
    });

    var langBtn = document.getElementById('lang-toggle');
    if (langBtn) langBtn.textContent = lang === 'zh' ? 'EN' : '中';

    buildGallery();
  }

  // ================================================================
  // GALLERY — Tab + Carousel
  // ================================================================
  var currentTab = 'all';

  function getCatCount(catId) {
    if (catId === 'all') return GAMES.length;
    return GAMES.filter(function (g) { return g.cat === catId; }).length;
  }

  function buildGallery() {
    var container = document.getElementById('gallery-dynamic');
    if (!container) return;

    var lang = currentLang;
    var html = '';

    // Tab bar
    html += '<div class="gallery-tabs">';
    CATEGORIES.forEach(function (cat) {
      var count = getCatCount(cat.id);
      var label = lang === 'zh' ? cat.zh : cat.en;
      var active = cat.id === currentTab ? ' active' : '';
      html += '<button class="gallery-tab' + active + '" data-cat="' + cat.id + '">' + label + ' (' + count + ')</button>';
    });
    html += '</div>';

    // Carousel for active tab
    CATEGORIES.forEach(function (cat) {
      if (cat.id !== currentTab) return;
      var games = cat.id === 'all' ? GAMES : GAMES.filter(function (g) { return g.cat === cat.id; });
      var catLabel = lang === 'zh' ? cat.zh : cat.en;

      html += '<div class="carousel-wrapper">';
      html += '<button class="carousel-btn carousel-prev" aria-label="Previous">&#8249;</button>';
      html += '<div class="carousel-viewport">';
      html += '<div class="carousel-track" data-cat="' + cat.id + '">';

      games.forEach(function (g) {
        var name = lang === 'zh' ? g.zh : g.en;
        var nameSub = lang === 'zh' ? g.en : g.zh;
        html += '<div class="carousel-card">';
        html += '  <img src="' + g.img + '" alt="' + g.en + '" loading="lazy">';
        html += '  <div class="carousel-card-overlay">';
        html += '    <span class="gallery-tag">' + catLabel + '</span>';
        html += '    <h4>' + name + '</h4>';
        html += '    <p>' + nameSub + '</p>';
        html += '  </div>';
        html += '</div>';
      });

      html += '</div>';
      html += '</div>';
      html += '<button class="carousel-btn carousel-next" aria-label="Next">&#8250;</button>';

      var cardsPerView = window.innerWidth > 768 ? 4 : (window.innerWidth > 480 ? 2 : 1);
      var totalPages = Math.ceil(games.length / cardsPerView);
      html += '<div class="carousel-dots">';
      for (var d = 0; d < totalPages; d++) {
        html += '<span class="carousel-dot' + (d === 0 ? ' active' : '') + '" data-page="' + d + '"></span>';
      }
      html += '</div>';
      html += '</div>';
    });

    container.innerHTML = html;

    // Bind tab clicks
    container.querySelectorAll('.gallery-tab').forEach(function (tab) {
      tab.addEventListener('click', function () {
        currentTab = tab.getAttribute('data-cat');
        buildGallery();
      });
    });

    initCarousel();
  }

  function initCarousel() {
    document.querySelectorAll('.carousel-wrapper').forEach(function (wrapper) {
      var viewport = wrapper.querySelector('.carousel-viewport');
      var track = wrapper.querySelector('.carousel-track');
      var prevBtn = wrapper.querySelector('.carousel-prev');
      var nextBtn = wrapper.querySelector('.carousel-next');
      var dots = wrapper.querySelectorAll('.carousel-dot');
      if (!viewport || !track) return;

      var page = 0;

      function getCardsPerView() {
        return window.innerWidth > 768 ? 4 : (window.innerWidth > 480 ? 2 : 1);
      }

      function getTotalPages() {
        var cards = track.querySelectorAll('.carousel-card');
        return Math.ceil(cards.length / getCardsPerView());
      }

      function goTo(p) {
        var total = getTotalPages();
        page = Math.max(0, Math.min(p, total - 1));
        var cpv = getCardsPerView();
        var card = track.querySelector('.carousel-card');
        var gap = parseFloat(window.getComputedStyle(track).columnGap) || 0;
        var pageWidth = card ? cpv * (card.offsetWidth + gap) : viewport.offsetWidth;
        var maxOffset = Math.max(0, track.scrollWidth - viewport.clientWidth);
        var offset = Math.min(page * pageWidth, maxOffset);
        track.style.transform = 'translateX(-' + offset + 'px)';

        dots.forEach(function (d, i) {
          d.classList.toggle('active', i === page);
        });
      }

      if (prevBtn) prevBtn.addEventListener('click', function () { goTo(page - 1); });
      if (nextBtn) nextBtn.addEventListener('click', function () { goTo(page + 1); });

      dots.forEach(function (dot) {
        dot.addEventListener('click', function () {
          goTo(parseInt(dot.getAttribute('data-page'), 10));
        });
      });

      // Touch/swipe
      var startX = 0;
      var isDragging = false;
      viewport.addEventListener('touchstart', function (e) {
        startX = e.touches[0].clientX;
        isDragging = true;
      }, { passive: true });
      viewport.addEventListener('touchend', function (e) {
        if (!isDragging) return;
        isDragging = false;
        var diff = startX - e.changedTouches[0].clientX;
        if (Math.abs(diff) > 50) {
          goTo(page + (diff > 0 ? 1 : -1));
        }
      }, { passive: true });
    });
  }

  // ================================================================
  // NAVBAR — Scroll effect
  // ================================================================
  var navbar = document.getElementById('navbar');

  function onScroll() {
    navbar.classList.toggle('scrolled', window.scrollY > 50);
  }

  window.addEventListener('scroll', onScroll, { passive: true });
  onScroll();

  // ---- Mobile nav toggle ----
  var toggle = document.querySelector('.nav-toggle');
  var navLinks = document.querySelector('.nav-links');

  if (toggle && navLinks) {
    toggle.addEventListener('click', function () {
      navLinks.classList.toggle('open');
    });
    navLinks.querySelectorAll('a').forEach(function (a) {
      a.addEventListener('click', function () { navLinks.classList.remove('open'); });
    });
  }

  // ================================================================
  // SCROLL REVEAL — Intersection Observer
  // ================================================================
  var fadeEls = document.querySelectorAll('.fade-in-up');

  if ('IntersectionObserver' in window) {
    var observer = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) {
          entry.target.classList.add('visible');
          observer.unobserve(entry.target);
        }
      });
    }, { threshold: 0.1, rootMargin: '0px 0px -40px 0px' });

    fadeEls.forEach(function (el) { observer.observe(el); });
  } else {
    fadeEls.forEach(function (el) { el.classList.add('visible'); });
  }

  // ================================================================
  // ANIMATED COUNTER
  // ================================================================
  var statNumbers = document.querySelectorAll('.stat-number[data-target]');

  function animateCounter(el) {
    var target = parseInt(el.dataset.target, 10);
    var suffix = el.dataset.suffix || '';
    var duration = 1800;
    var start = performance.now();

    function tick(now) {
      var elapsed = now - start;
      var progress = Math.min(elapsed / duration, 1);
      var eased = 1 - Math.pow(1 - progress, 3);
      var current = Math.round(eased * target);
      el.textContent = current.toLocaleString() + suffix;
      if (progress < 1) requestAnimationFrame(tick);
    }

    requestAnimationFrame(tick);
  }

  if ('IntersectionObserver' in window) {
    var statObserver = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) {
          animateCounter(entry.target);
          statObserver.unobserve(entry.target);
        }
      });
    }, { threshold: 0.5 });

    statNumbers.forEach(function (el) { statObserver.observe(el); });
  } else {
    statNumbers.forEach(function (el) { animateCounter(el); });
  }

  // ================================================================
  // CIRCUIT CANVAS — Hero background (green particles)
  // ================================================================
  var canvas = document.getElementById('circuit-canvas');
  if (canvas && canvas.getContext) {
    var ctx = canvas.getContext('2d');
    var w, h, particles;
    var PARTICLE_COUNT = 60;
    var LINE_DIST = 120;

    function resize() {
      w = canvas.width = canvas.offsetWidth;
      h = canvas.height = canvas.offsetHeight;
    }

    function initParticles() {
      particles = [];
      for (var i = 0; i < PARTICLE_COUNT; i++) {
        particles.push({
          x: Math.random() * w,
          y: Math.random() * h,
          vx: (Math.random() - 0.5) * 0.4,
          vy: (Math.random() - 0.5) * 0.4,
          r: Math.random() * 2 + 1
        });
      }
    }

    function draw() {
      ctx.clearRect(0, 0, w, h);

      for (var i = 0; i < particles.length; i++) {
        for (var j = i + 1; j < particles.length; j++) {
          var dx = particles[i].x - particles[j].x;
          var dy = particles[i].y - particles[j].y;
          var dist = Math.sqrt(dx * dx + dy * dy);
          if (dist < LINE_DIST) {
            var alpha = (1 - dist / LINE_DIST) * 0.4;
            ctx.strokeStyle = 'rgba(0, 255, 65, ' + alpha + ')';
            ctx.lineWidth = 0.5;
            ctx.beginPath();
            ctx.moveTo(particles[i].x, particles[i].y);
            ctx.lineTo(particles[j].x, particles[j].y);
            ctx.stroke();
          }
        }
      }

      particles.forEach(function (p) {
        ctx.fillStyle = 'rgba(0, 204, 51, 0.6)';
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx.fill();

        p.x += p.vx;
        p.y += p.vy;

        if (p.x < 0) p.x = w;
        if (p.x > w) p.x = 0;
        if (p.y < 0) p.y = h;
        if (p.y > h) p.y = 0;
      });

      requestAnimationFrame(draw);
    }

    resize();
    initParticles();
    draw();

    window.addEventListener('resize', function () {
      resize();
      initParticles();
    });
  }

  // ================================================================
  // SMOOTH SCROLL
  // ================================================================
  document.querySelectorAll('a[href^="#"]').forEach(function (anchor) {
    anchor.addEventListener('click', function (e) {
      var target = document.querySelector(anchor.getAttribute('href'));
      if (target) {
        e.preventDefault();
        target.scrollIntoView({ behavior: 'smooth', block: 'start' });
      }
    });
  });

  // ================================================================
  // LANGUAGE TOGGLE
  // ================================================================
  var langBtn = document.getElementById('lang-toggle');
  if (langBtn) {
    langBtn.addEventListener('click', function () {
      applyLang(currentLang === 'zh' ? 'en' : 'zh');
    });
  }

  // ================================================================
  // INIT
  // ================================================================
  applyLang(currentLang);
  buildGallery();

  // Re-init scroll reveal for dynamically added elements
  if ('IntersectionObserver' in window) {
    var revealObserver = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) {
          entry.target.classList.add('visible');
          revealObserver.unobserve(entry.target);
        }
      });
    }, { threshold: 0.1, rootMargin: '0px 0px -40px 0px' });

    setTimeout(function () {
      document.querySelectorAll('.fade-in-up:not(.visible)').forEach(function (el) {
        revealObserver.observe(el);
      });
    }, 100);
  }

})();
