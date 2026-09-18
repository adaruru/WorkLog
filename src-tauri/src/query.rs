use rusqlite::types::Value;

use crate::models::{EntryQuery, UserFilter};

const DEFAULT_STATUS: &str =
    "s.id = (SELECT CAST(value AS INTEGER) FROM app_settings WHERE key = 'default_status_id')";

fn sort_expressions(field: &str) -> (&'static str, &'static [&'static str]) {
    match field {
        "ticket" => (
            "IFNULL(e.ticket, '') = ''",
            &[
                "CAST(IFNULL(e.ticket, '') AS INTEGER)",
                "IFNULL(e.ticket, '')",
            ],
        ),
        "title" => ("TRIM(e.title) = ''", &["e.title"]),
        "hours" => ("e.hours = 0", &["e.hours"]),
        "status" => (DEFAULT_STATUS, &["s.sort_order"]),
        "user" => ("e.user_id IS NULL", &["IFNULL(u.name, '')"]),
        _ => ("e.work_date IS NULL", &["e.work_date"]),
    }
}

pub fn build_order(query: &EntryQuery) -> String {
    let (blank, columns) = sort_expressions(&query.sort_field);
    let direction = if query.sort_dir.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };
    let ordered = columns
        .iter()
        .map(|column| format!("{column} {direction}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(" ORDER BY {blank} ASC, {ordered}, e.updated_at DESC")
}

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
    fn default_order_is_work_date_descending() {
        let sql = build_order(&EntryQuery::default());
        assert_eq!(sql, " ORDER BY e.work_date IS NULL ASC, e.work_date DESC, e.updated_at DESC");
    }

    #[test]
    fn each_sortable_column_maps_to_its_expression() {
        let cases = [
            ("title", "e.title"),
            ("hours", "e.hours"),
            ("status", "s.sort_order"),
            ("user", "IFNULL(u.name, '')"),
            ("work_date", "e.work_date"),
        ];
        for (field, column) in cases {
            let query = EntryQuery {
                sort_field: field.to_string(),
                sort_dir: "asc".to_string(),
                ..EntryQuery::default()
            };
            assert!(
                build_order(&query).contains(&format!("{column} ASC, e.updated_at DESC")),
                "欄位 {field} 的排序運算式不符"
            );
        }
    }

    #[test]
    fn a_numeric_ticket_sorts_by_value_not_by_text() {
        let query = EntryQuery {
            sort_field: "ticket".to_string(),
            sort_dir: "asc".to_string(),
            ..EntryQuery::default()
        };
        let sql = build_order(&query);
        assert!(sql.contains("CAST(IFNULL(e.ticket, '') AS INTEGER) ASC"));
        assert!(sql.contains("IFNULL(e.ticket, '') ASC"));
    }

    #[test]
    fn blank_values_sink_to_the_bottom_in_both_directions() {
        for direction in ["asc", "desc"] {
            let query = EntryQuery {
                sort_field: "ticket".to_string(),
                sort_dir: direction.to_string(),
                ..EntryQuery::default()
            };
            assert!(build_order(&query).starts_with(" ORDER BY IFNULL(e.ticket, '') = '' ASC,"));
        }
    }

    #[test]
    fn each_column_declares_what_counts_as_blank() {
        let cases = [
            ("ticket", "IFNULL(e.ticket, '') = ''"),
            ("title", "TRIM(e.title) = ''"),
            ("hours", "e.hours = 0"),
            ("user", "e.user_id IS NULL"),
        ];
        for (field, blank) in cases {
            let query = EntryQuery {
                sort_field: field.to_string(),
                ..EntryQuery::default()
            };
            assert!(build_order(&query).contains(&format!("ORDER BY {blank} ASC")));
        }
    }

    #[test]
    fn the_default_status_sinks_to_the_bottom() {
        let query = EntryQuery {
            sort_field: "status".to_string(),
            ..EntryQuery::default()
        };
        assert!(build_order(&query).contains("default_status_id"));
    }

    #[test]
    fn an_unknown_sort_field_falls_back_to_work_date() {
        let query = EntryQuery {
            sort_field: "e.title; DROP TABLE entries".to_string(),
            ..EntryQuery::default()
        };
        assert_eq!(
            build_order(&query),
            " ORDER BY e.work_date IS NULL ASC, e.work_date DESC, e.updated_at DESC"
        );
    }

    #[test]
    fn an_unknown_direction_falls_back_to_descending() {
        let query = EntryQuery {
            sort_field: "title".to_string(),
            sort_dir: "ASC; DROP TABLE entries".to_string(),
            ..EntryQuery::default()
        };
        assert!(build_order(&query).contains("e.title DESC"));
    }

    #[test]
    fn direction_is_case_insensitive() {
        let query = EntryQuery {
            sort_field: "hours".to_string(),
            sort_dir: "ASC".to_string(),
            ..EntryQuery::default()
        };
        assert!(build_order(&query).contains("e.hours ASC"));
    }

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
            sort_field: "work_date".to_string(),
            sort_dir: "desc".to_string(),
        };
        let (sql, params) = build_filter(&query);
        assert_eq!(sql.matches(" AND ").count(), 4);
        assert_eq!(params.len(), 7);
    }
}
