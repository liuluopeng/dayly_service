export function useSearchHistory() {
  const addSearchHistory = (term: string) => {
    if (!term.trim()) return;

    const savedHistory = localStorage.getItem('searchHistory');
    let history = [];

    if (savedHistory) {
      try {
        history = JSON.parse(savedHistory);
      } catch (e) {
        console.error('Failed to parse search history:', e);
        history = [];
      }
    }

    // 移除重复项
    history = history.filter((item: any) => item.term !== term);

    // 添加新搜索项到开头
    history.unshift({
      term,
      timestamp: Date.now()
    });

    // 限制历史记录数量为100条
    if (history.length > 100) {
      history = history.slice(0, 100);
    }

    localStorage.setItem('searchHistory', JSON.stringify(history));
  };

  return {
    addSearchHistory
  };
}
