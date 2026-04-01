-- Dashboard query for highlighted SQL coverage.
WITH recent_themes AS (
  SELECT
    t.id,
    t.name,
    COUNT(p.id) AS preview_count
  FROM themes AS t
  LEFT JOIN previews AS p
    ON p.theme_id = t.id
  WHERE t.enabled = true
    AND t.kind = $1
    AND t.note = $kat$dracula$kat$
  GROUP BY t.id, t.name
)
SELECT
  name,
  preview_count,
  CASE
    WHEN preview_count > 10 THEN true
    ELSE false
  END AS featured
FROM recent_themes
ORDER BY preview_count DESC
LIMIT 5;
