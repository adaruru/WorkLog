use rusqlite::types::Value;

use crate::models::{EntryQuery, UserFilter};

pub const ORDER_BY: &str = " ORDER BY e.work_date DESC, e.updated_at DESC";

pub fn build_filter(query: &EntryQuery) -> (String, Vec<Value>) {
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<Value> = Vec::new();

    let keyword = query.keyword.trim();
    if !keyword.is_empty() {
        clauses.push(
            "(e.title LIKE ? ESCAPE '\\' OR IFNULL(e.content, '') LIKE ? ESCAPE '\\' OR IFNULL(e.ticket, '') LIKE ? ESCAPE '\\')"
                .to_string(),
        );
        let like = format!("%{}%", escape_like(keyword));
        params.push(Value::Text(like.clone()));
        params.push(Value::Text(like.clone()));
        params.push(Value::Text(like));
    }

    let from = query.date_from.trim();
    if !from.is_empty() {
        clauses.push("e.work_date >= ?".to_string());
        params.push(Value::Text(from.to_string()));
    }

    let to = query.date_to.trim();
    if !to.is_empty() {
        clauses.push("e.work_date <= ?".to_string());
        params.push(Value::Text(to.to_string()));
    }

    if !query.status_ids.is_empty() {
        let holders = vec!["?"; query.status_ids.len()].join(", ");
        clauses.push(format!("e.status_id IN ({holders})"));
        for id in &query.status_ids {
            params.push(Value::Integer(*id));
        }
    }

    match query.user_filter {
        UserFilter::Unowned => clauses.push("e.user_id IS NULL".to_string()),
        UserFilter::One => {
            if let Some(user_id) = query.user_id {
                clauses.push("e.user_id = ?".to_string());
                params.push(Value::Integer(user_id));
            }
        }
        UserFilter::Any => {}
    }

    let where_sql = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };

    (where_sql, params)
}

pub fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_produces_no_where_clause() {
        let (sql, params) = build_filter(&EntryQuery::default());
        assert!(sql.is_empty());
        assert!(params.is_empty());
    }

    #[test]
    fn blank_strings_are_treated_as_absent() {
        let query = EntryQuery {
            keyword: "   ".to_string(),
            date_from: "".to_string(),
            date_to: "  ".to_string(),
            ..EntryQuery::default()
        };
        let (sql, params) = build_filter(&query);
        assert!(sql.is_empty());
        assert!(params.is_empty());
    }

    #[test]
    fn keyword_hits_title_content_and_ticket() {
        let query = EntryQuery {
            keyword: "登入".to_string(),
            ..EntryQuery::default()
        };
        let (sql, params) = build_filter(&query);
        assert!(sql.contains("e.title LIKE"));
        assert!(sql.contains("e.content"));
        assert!(sql.contains("e.ticket"));
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], Value::Text("%登入%".to_string()));
    }

    #[test]
    fn keyword_wildcards_are_escaped() {
        let query = EntryQuery {
            keyword: "100%_done".to_string(),
            ..EntryQuery::default()
        };
        let (_, params) = build_filter(&query);
        assert_eq!(params[0], Value::Text("%100\\%\\_done%".to_string()));
    }

    #[test]
    fn only_the_supplied_date_bound_is_applied() {
        let query = EntryQuery {
            date_from: "2026-09-01".to_string(),
            ..EntryQuery::default()
        };
        let (sql, params) = build_filter(&query);
        assert!(sql.contains("e.work_date >= ?"));
        assert!(!sql.contains("e.work_date <= ?"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn status_ids_expand_to_matching_placeholders() {
        let query = EntryQuery {
            status_ids: vec![1, 2, 5],
            ..EntryQuery::default()
        };
        let (sql, params) = build_filter(&query);
        assert!(sql.contains("e.status_id IN (?, ?, ?)"));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn unowned_filter_needs_no_parameter() {
        let query = EntryQuery {
            user_filter: UserFilter::Unowned,
            ..EntryQuery::default()
        };
        let (sql, params) = build_filter(&query);
        assert!(sql.contains("e.user_id IS NULL"));
        assert!(params.is_empty());
    }

    #[test]
    fn single_user_filter_binds_the_id() {
        let query = EntryQuery {
            user_filter: UserFilter::One,
            user_id: Some(7),
            ..EntryQuery::default()
        };
        let (sql, params) = build_filter(&query);
        assert!(sql.contains("e.user_id = ?"));
        assert_eq!(params, vec![Value::Integer(7)]);
    }

    #[test]
    fn single_user_filter_without_id_falls_back_to_all_users() {
        let query = EntryQuery {
            user_filter: UserFilter::One,
            user_id: None,
            ..EntryQuery::default()
        };
        let (sql, params) = build_filter(&query);
        assert!(sql.is_empty());
        assert!(params.is_empty());
    }

    #[test]
    fn conditions_are_joined_with_and() {
        let query = EntryQuery {
            keyword: "bug".to_string(),
            date_from: "2026-09-01".to_string(),
            date_to: "2026-09-30".to_string(),
            status_ids: vec![3],
            user_filter: UserFilter::One,
            user_id: Some(2),
        };
        let (sql, params) = build_filter(&query);
        assert_eq!(sql.matches(" AND ").count(), 4);
        assert_eq!(params.len(), 7);
    }
}
