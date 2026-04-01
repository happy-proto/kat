-- Rich SQL showcase for direct-file highlighting.
WITH ranked AS (
  SELECT
    t.id,
    t.slug,
    t.name,
    COUNT(p.id) AS preview_count,
    ROW_NUMBER() OVER (
      PARTITION BY t.kind
      ORDER BY COUNT(p.id) DESC
    ) AS rank_in_kind
  FROM themes AS t
  LEFT JOIN previews AS p
    ON p.theme_id = t.id
  WHERE t.enabled = true
    AND t.kind = $1
    AND t.metadata = $json$
      {"accent":"#ff79c6","featured":true}
    $json$
  GROUP BY t.id, t.slug, t.name, t.kind
)
SELECT
  slug,
  name,
  preview_count,
  rank_in_kind,
  CASE
    WHEN preview_count >= 10 THEN 'spotlight'
    ELSE 'standard'
  END AS bucket
FROM ranked
WHERE rank_in_kind <= 3
ORDER BY preview_count DESC, name ASC;
