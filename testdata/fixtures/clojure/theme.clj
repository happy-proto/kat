(ns kat.theme)

(def theme {:theme/name "Dracula"})

(defn render-theme []
  (println (:theme/name theme)))
