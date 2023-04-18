#ifndef BVG_H
#define BVG_H

#include <vector>
#include <memory>
#include <algorithm>

#include "hittable.h"
#include "aabb.h"

class BVHNode : public Hittable
{
public:
    BVHNode() {}
    BVHNode(std::vector<std::shared_ptr<Hittable>> &objects, size_t start, size_t end, double time0, double time1);

    virtual bool hit(const Ray &r, double t_min, double t_max, HitRecord &rec) const override;
    virtual bool bounding_box(double time0, double time1, AABB &output_box) const override;
    std::string toString() const override { return "BVHNode"; }
public:
    std::shared_ptr<Hittable> left;
    std::shared_ptr<Hittable> right;
    AABB box;
};

bool box_compare(const std::shared_ptr<Hittable> a, const std::shared_ptr<Hittable> b, int axis);
bool box_x_compare(const std::shared_ptr<Hittable> a, const std::shared_ptr<Hittable> b);
bool box_y_compare(const std::shared_ptr<Hittable> a, const std::shared_ptr<Hittable> b);
bool box_z_compare(const std::shared_ptr<Hittable> a, const std::shared_ptr<Hittable> b);

#endif // BVG_H